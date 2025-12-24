//! Agent management service.
//!
//! Handles CRUD operations for agents, including key generation and quota checks.

use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::{
    log_audit_event, Agent, AgentResponse, CreateAgentRequest, CreateAgentResponse,
    PaginatedResponse, QuotaUsage, UpdateAgentRequest,
};

use crate::services::jwt::JwtService;
use crate::services::quota::QuotaService;
use crate::utils::api_key::ApiKeyGenerator;

/// Service for managing agents.
pub struct AgentService {
    _jwt_service: Arc<JwtService>,
}

impl AgentService {
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        Self { _jwt_service: jwt_service }
    }

    /// Create a new agent.
    pub async fn create_agent(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        created_by: Uuid,
        request: CreateAgentRequest,
    ) -> Result<CreateAgentResponse, ApiError> {
        // 1. Check team quota
        if !QuotaService::check_agent_limit(pool, team_id).await? {
            return Err(ApiError::Conflict(
                "Team agent limit reached. Upgrade your plan to create more agents.".to_string(),
            ));
        }

        // 2. Create agent
        let (agent, api_key) = Agent::create(
            pool,
            team_id,
            &request.name,
            request.description,
            created_by,
        )
        .await?;

        // 3. Initialize quota
        QuotaService::initialize_agent_quota(pool, agent.id, team_id).await?;

        // 4. Log audit event
        log_audit_event(
            pool,
            team_id,
            Some(created_by),
            "agent.create",
            Some("agent"),
            Some(agent.id),
            Some(&format!("Created agent '{}'", agent.name)),
            None,
        )
        .await?;

        Ok(CreateAgentResponse {
            agent: agent.to_response(),
            api_key,
            warning: "Save this API key - it won't be shown again!".to_string(),
        })
    }

    /// Get an agent by ID.
    pub async fn get_agent(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        agent_id: Uuid,
    ) -> Result<AgentResponse, ApiError> {
        let agent = Agent::find_by_id(pool, agent_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

        if agent.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this agent".to_string()));
        }

        Ok(agent.to_response())
    }

    /// List agents for a team.
    pub async fn list_agents(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<PaginatedResponse<AgentResponse>, ApiError> {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);

        let (agents, total) = Agent::find_by_team(pool, team_id, page, limit).await?;
        
        let total_pages = (total as f64 / limit as f64).ceil() as i32;

        Ok(PaginatedResponse {
            data: agents.into_iter().map(|a| a.to_response()).collect(),
            total,
            page,
            limit,
            pages: total_pages,
        })
    }

    /// Update an agent.
    pub async fn update_agent(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        agent_id: Uuid,
        request: UpdateAgentRequest,
    ) -> Result<AgentResponse, ApiError> {
        let agent = Agent::find_by_id(pool, agent_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

        if agent.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this agent".to_string()));
        }

        let updated_agent = Agent::update(
            pool,
            agent_id,
            request.name.clone(),
            request.description.clone(),
            request.status.clone(),
        )
        .await?;

        let mut changes = Vec::new();
        if request.name.is_some() { changes.push("name"); }
        if request.description.is_some() { changes.push("description"); }
        if request.status.is_some() { changes.push("status"); }
        let desc = format!("Updated fields: {}", changes.join(", "));

        log_audit_event(
            pool,
            team_id,
            None,
            "agent.update",
            Some("agent"),
            Some(agent_id),
            Some(&desc),
            None,
        )
        .await?;

        Ok(updated_agent.to_response())
    }

    /// Delete an agent.
    pub async fn delete_agent(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        agent_id: Uuid,
    ) -> Result<(), ApiError> {
        let agent = Agent::find_by_id(pool, agent_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

        if agent.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this agent".to_string()));
        }

        Agent::soft_delete(pool, agent_id).await?;

        log_audit_event(
            pool,
            team_id,
            None,
            "agent.delete",
            Some("agent"),
            Some(agent_id),
            Some("Soft deleted agent"),
            None,
        )
        .await?;

        Ok(())
    }

    /// Authenticate agent by API key.
    pub async fn get_agent_by_api_key(
        &self,
        pool: &PgPool,
        api_key: &str,
    ) -> Result<Agent, ApiError> {
        if !ApiKeyGenerator::validate_format(api_key) {
            return Err(ApiError::Unauthorized("Invalid API key format".to_string()));
        }

        let hash = ApiKeyGenerator::hash(api_key);
        let agent = Agent::find_by_api_key_hash(pool, &hash)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("Invalid API key".to_string()))?;

        // Update usage stats (optional, could be async background)
        // Here we do it synchronously for simplicity
        Agent::update_last_used(pool, agent.id).await?;
        
        Ok(agent)
    }

    /// Get usage stats.
    pub async fn get_usage_stats(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        agent_id: Uuid,
    ) -> Result<QuotaUsage, ApiError> {
        let agent = Agent::find_by_id(pool, agent_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

        if agent.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied".to_string()));
        }

        QuotaService::get_quota_usage(pool, agent_id).await
    }
    
    /// Verify API key and return (agent_id, team_id).
    pub async fn verify_api_key(
        &self,
        pool: &PgPool,
        api_key: &str,
    ) -> Result<(Uuid, Uuid), ApiError> {
        if !ApiKeyGenerator::validate_format(api_key) {
             return Err(ApiError::Unauthorized("Invalid API key format".to_string()));
        }

        // We use get_agent_by_api_key which also updates last_used
        let agent = self.get_agent_by_api_key(pool, api_key).await?;
        
        Ok((agent.id, agent.team_id))
    }
}
