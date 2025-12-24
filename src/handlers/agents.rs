//! Agent management handlers.

use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::api_key::ApiKeyAuth;
use crate::middleware::auth::AuthUser;
use crate::models::{CreateAgentRequest, PaginationQuery, UpdateAgentRequest};
use crate::services::agent::AgentService;

/// Configure agent routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_agent);
    cfg.service(list_agents);
    cfg.service(get_agent);
    cfg.service(update_agent);
    cfg.service(delete_agent);
    cfg.service(get_agent_usage);
    cfg.service(check_agent_status);
}

/// Create a new agent.
#[post("")]
async fn create_agent(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    request: web::Json<CreateAgentRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .create_agent(&pool, auth.team_id, auth.user_id, request.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(response))
}

/// List agents.
#[get("")]
async fn list_agents(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let response = service
        .list_agents(&pool, auth.team_id, query.page, query.limit)
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Get agent details.
#[get("/{id}")]
async fn get_agent(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_agent(&pool, auth.team_id, path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Update agent.
#[patch("/{id}")]
async fn update_agent(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateAgentRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .update_agent(&pool, auth.team_id, path.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Delete agent.
#[delete("/{id}")]
async fn delete_agent(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    service
        .delete_agent(&pool, auth.team_id, path.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get agent usage stats.
#[get("/{id}/usage")]
async fn get_agent_usage(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<AgentService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_usage_stats(&pool, auth.team_id, path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Check agent status (authenticated via API Key).
#[get("/{id}/status")]
async fn check_agent_status(
    auth: ApiKeyAuth,
    path: web::Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let agent_id = path.into_inner();
    
    // Verify the authenticated agent matches the requested ID
    if auth.agent_id != agent_id {
        return Err(ApiError::Forbidden("Token does not match agent ID".to_string()));
    }

    // If we got here, the agent is active and authenticated
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "active",
        "agent_id": agent_id,
        "team_id": auth.team_id
    })))
}
