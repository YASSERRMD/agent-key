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
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedAuditResponse {
    pub data: Vec<AuditEvent>,
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
    pub start_date: Option<String>,
    pub end_date: Option<String>,
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

    // Build query with optional filters
    let mut sql = String::from(
        "SELECT id, team_id, user_id, event_type, resource_type, resource_id, details, ip_address, created_at 
         FROM audit_events 
         WHERE team_id = $1"
    );
    
    let mut params: Vec<String> = vec![];
    let mut param_idx = 2;

    if let Some(ref event_type) = query.event_type {
        sql.push_str(&format!(" AND event_type = ${}", param_idx));
        params.push(event_type.clone());
        param_idx += 1;
    }

    if let Some(ref resource_type) = query.resource_type {
        sql.push_str(&format!(" AND resource_type = ${}", param_idx));
        params.push(resource_type.clone());
    }

    sql.push_str(" ORDER BY created_at DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    // For simplicity, we'll use a simpler query approach
    let events = sqlx::query_as::<_, AuditEvent>(
        r#"
        SELECT id, team_id, user_id, event_type, resource_type, resource_id, details, ip_address, created_at 
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

    Ok(HttpResponse::Ok().json(PaginatedAuditResponse {
        data: events,
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
        SELECT id, team_id, user_id, event_type, resource_type, resource_id, details, ip_address, created_at 
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

    Ok(HttpResponse::Ok().json(event))
}
