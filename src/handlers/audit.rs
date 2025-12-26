//! Audit log handlers.

use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;

/// Configure audit routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/audit")
            .service(list_audit_events)
            .service(get_audit_event)
    );
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditEvent {
    pub id: i64,
    pub team_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub change_description: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuditEventResponse {
    pub id: i64,
    pub team_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditEvent> for AuditEventResponse {
    fn from(e: AuditEvent) -> Self {
        AuditEventResponse {
            id: e.id,
            team_id: e.team_id,
            user_id: e.user_id,
            event_type: e.event_type,
            resource_type: e.resource_type,
            resource_id: e.resource_id.map(|id| id.to_string()),
            details: e.change_description,
            ip_address: e.ip_address,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedAuditResponse {
    pub data: Vec<AuditEventResponse>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
    pub pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
}

/// GET /api/v1/audit
/// List audit events for the team.
#[get("")]
pub async fn list_audit_events(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    query: web::Query<AuditQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let events = sqlx::query_as::<_, AuditEvent>(
        r#"
        SELECT id, team_id, user_id, event_type, resource_type, resource_id, change_description, ip_address, created_at 
        FROM audit_events 
        WHERE team_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(auth.team_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_events WHERE team_id = $1"
    )
    .bind(auth.team_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let pages = ((total.0 as f64) / (limit as f64)).ceil() as i32;

    let response_data: Vec<AuditEventResponse> = events.into_iter().map(|e| e.into()).collect();

    Ok(HttpResponse::Ok().json(PaginatedAuditResponse {
        data: response_data,
        total: total.0,
        page,
        limit,
        pages,
    }))
}

/// GET /api/v1/audit/{id}
/// Get a specific audit event.
#[get("/{id}")]
pub async fn get_audit_event(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    let event = sqlx::query_as::<_, AuditEvent>(
        r#"
        SELECT id, team_id, user_id, event_type, resource_type, resource_id, change_description, ip_address, created_at 
        FROM audit_events 
        WHERE id = $1 AND team_id = $2
        "#,
    )
    .bind(id)
    .bind(auth.team_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Audit event not found".to_string()))?;

    let response: AuditEventResponse = event.into();
    Ok(HttpResponse::Ok().json(response))
}

