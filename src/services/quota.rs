//! Quota management service.
//!
//! Handles quota enforcement for agents and teams including API call limits and agent counts.

use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::{AgentQuota, QuotaMetric, QuotaUsage, Team};

/// Service for managing and enforcing quotas.
pub struct QuotaService;

impl QuotaService {
    /// Check if a team can create more agents.
    pub async fn check_agent_limit(pool: &PgPool, team_id: Uuid) -> Result<bool, ApiError> {
        let team = Team::find_by_id(pool, team_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Team not found".to_string()))?;

        team.check_agent_quota(pool).await
    }

    /// Check if an agent has remaining API call quota for the current month.
    pub async fn check_api_call_quota(pool: &PgPool, agent_id: Uuid) -> Result<bool, ApiError> {
        let month_year = Self::get_current_month_year();
        
        let quota = sqlx::query_as::<_, AgentQuota>(
            r#"
            SELECT * FROM agent_quotas 
            WHERE agent_id = $1 AND month_year = $2
            "#,
        )
        .bind(agent_id)
        .bind(&month_year)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if let Some(q) = quota {
            if q.api_calls_limit == -1 {
                return Ok(true); // Unlimited
            }
            Ok(q.api_calls_used < q.api_calls_limit)
        } else {
            // No quota record implies no quota initialized, assuming blocked or need init
            // For now, let's assume if no record, we should create one or return false
            // But typical flow is quota created on agent creation or monthly job
            // If missing, we'll return true to be safe? No, should be strict.
            // But let's return false and log error usually.
            // However, for this implementation, let's assume explicit quota required.
            Ok(false)
        }
    }

    /// Increment API call usage for an agent.
    pub async fn increment_api_calls(pool: &PgPool, agent_id: Uuid) -> Result<(), ApiError> {
        let month_year = Self::get_current_month_year();

        sqlx::query(
            r#"
            UPDATE agent_quotas 
            SET api_calls_used = api_calls_used + 1, updated_at = CURRENT_TIMESTAMP
            WHERE agent_id = $1 AND month_year = $2
            "#,
        )
        .bind(agent_id)
        .bind(&month_year)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get usage statistics for an agent.
    pub async fn get_quota_usage(pool: &PgPool, agent_id: Uuid) -> Result<QuotaUsage, ApiError> {
        let month_year = Self::get_current_month_year();

        let quota = sqlx::query_as::<_, AgentQuota>(
            r#"
            SELECT * FROM agent_quotas 
            WHERE agent_id = $1 AND month_year = $2
            "#,
        )
        .bind(agent_id)
        .bind(&month_year)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Quota not found for current month".to_string()))?;

        Ok(QuotaUsage {
            month: month_year,
            api_calls: QuotaMetric {
                used: quota.api_calls_used,
                limit: quota.api_calls_limit,
                percentage: if quota.api_calls_limit > 0 {
                    (quota.api_calls_used as f32 / quota.api_calls_limit as f32) * 100.0
                } else if quota.api_calls_limit == -1 {
                    0.0 // Unlimited
                } else {
                    100.0 // Limit 0
                },
            },
            key_rotations: QuotaMetric {
                used: quota.key_rotations_used,
                limit: quota.key_rotations_limit,
                percentage: if quota.key_rotations_limit > 0 {
                    (quota.key_rotations_used as f32 / quota.key_rotations_limit as f32) * 100.0
                } else {
                    0.0
                },
            },
        })
    }

    /// Initialize quota for a new agent.
    pub async fn initialize_agent_quota(
        pool: &PgPool,
        agent_id: Uuid,
        team_id: Uuid,
    ) -> Result<(), ApiError> {
        let team = Team::find_by_id(pool, team_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Team not found".to_string()))?;

        let month_year = Self::get_current_month_year();
        
        // Determine limits based on plan
        // This logic could be moved to a PlanService or config
        let (api_limit, rotation_limit) = match team.plan.as_str() {
            "enterprise" => (-1, 100), // -1 for unlimited
            "pro" => (100_000, 50),
            _ => (1_000, 5), // Free
        };

        sqlx::query(
            r#"
            INSERT INTO agent_quotas (agent_id, team_id, month_year, api_calls_limit, key_rotations_limit)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(agent_id)
        .bind(team_id)
        .bind(&month_year)
        .bind(api_limit)
        .bind(rotation_limit)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Helper to get current "YYYY-MM".
    fn get_current_month_year() -> String {
        let now = Utc::now();
        format!("{:04}-{:02}", now.year(), now.month())
    }
}
