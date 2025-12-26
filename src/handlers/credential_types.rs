//! Credential types handler for managing configurable credential types.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;

/// Configure credential types routes.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/credential-types")
            .route("", web::get().to(list_credential_types))
            .route("", web::post().to(create_credential_type))
            .route("/{id}", web::patch().to(update_credential_type))
            .route("/{id}", web::delete().to(delete_credential_type)),
    );
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CredentialType {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCredentialTypeRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCredentialTypeRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// List all credential types for the team.
/// GET /api/v1/credential-types
async fn list_credential_types(
    auth: AuthUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let types = sqlx::query_as::<_, CredentialType>(
        r#"
        SELECT id, team_id, name, display_name, description, icon, color, is_system, created_at
        FROM credential_types
        WHERE team_id = $1 AND deleted_at IS NULL
        ORDER BY is_system DESC, display_name ASC
        "#,
    )
    .bind(auth.team_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(types))
}

/// Create a new credential type.
/// POST /api/v1/credential-types
async fn create_credential_type(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    body: web::Json<CreateCredentialTypeRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate name format (lowercase, alphanumeric with underscores)
    let name = body.name.to_lowercase().replace(" ", "_");
    if name.is_empty() || name.len() > 50 {
        return Err(ApiError::ValidationError("Name must be 1-50 characters".to_string()));
    }

    let credential_type = sqlx::query_as::<_, CredentialType>(
        r#"
        INSERT INTO credential_types (team_id, name, display_name, description, icon, color, is_system)
        VALUES ($1, $2, $3, $4, $5, $6, FALSE)
        RETURNING id, team_id, name, display_name, description, icon, color, is_system, created_at
        "#,
    )
    .bind(auth.team_id)
    .bind(&name)
    .bind(&body.display_name)
    .bind(&body.description)
    .bind(&body.icon)
    .bind(&body.color)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            ApiError::ValidationError(format!("Credential type '{}' already exists", name))
        } else {
            ApiError::DatabaseError(e.to_string())
        }
    })?;

    Ok(HttpResponse::Created().json(credential_type))
}

/// Update a credential type.
/// PATCH /api/v1/credential-types/{id}
async fn update_credential_type(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCredentialTypeRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    // Check if exists and belongs to team
    let existing = sqlx::query_as::<_, CredentialType>(
        "SELECT id, team_id, name, display_name, description, icon, color, is_system, created_at 
         FROM credential_types WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Credential type not found".to_string()))?;

    if existing.team_id != auth.team_id {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    if existing.is_system {
        return Err(ApiError::ValidationError("Cannot modify system credential types".to_string()));
    }

    let updated = sqlx::query_as::<_, CredentialType>(
        r#"
        UPDATE credential_types SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            icon = COALESCE($4, icon),
            color = COALESCE($5, color),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, team_id, name, display_name, description, icon, color, is_system, created_at
        "#,
    )
    .bind(id)
    .bind(&body.display_name)
    .bind(&body.description)
    .bind(&body.icon)
    .bind(&body.color)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(updated))
}

/// Delete a credential type.
/// DELETE /api/v1/credential-types/{id}
async fn delete_credential_type(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    // Check if exists and belongs to team
    let existing = sqlx::query_as::<_, CredentialType>(
        "SELECT id, team_id, name, display_name, description, icon, color, is_system, created_at 
         FROM credential_types WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Credential type not found".to_string()))?;

    if existing.team_id != auth.team_id {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    if existing.is_system {
        return Err(ApiError::ValidationError("Cannot delete system credential types".to_string()));
    }

    // Check if any credentials are using this type
    let usage_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM credentials WHERE credential_type = $1 AND team_id = $2 AND deleted_at IS NULL",
    )
    .bind(&existing.name)
    .bind(auth.team_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if usage_count.0 > 0 {
        return Err(ApiError::ValidationError(format!(
            "Cannot delete: {} credentials are using this type",
            usage_count.0
        )));
    }

    sqlx::query("UPDATE credential_types SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

/// Seed default credential types for a new team.
pub async fn seed_default_types(pool: &PgPool, team_id: Uuid) -> Result<(), ApiError> {
    let defaults = vec![
        ("generic", "Generic", "General purpose credential", "key", "gray"),
        ("api_key", "API Key", "API authentication key", "key", "blue"),
        ("aws", "AWS", "Amazon Web Services credentials", "cloud", "orange"),
        ("openai", "OpenAI", "OpenAI API key", "brain", "green"),
        ("database", "Database", "Database connection credentials", "database", "purple"),
        ("oauth", "OAuth Token", "OAuth access token", "lock", "teal"),
    ];

    for (name, display_name, description, icon, color) in defaults {
        sqlx::query(
            r#"
            INSERT INTO credential_types (team_id, name, display_name, description, icon, color, is_system)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE)
            ON CONFLICT (team_id, name) DO NOTHING
            "#,
        )
        .bind(team_id)
        .bind(name)
        .bind(display_name)
        .bind(description)
        .bind(icon)
        .bind(color)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}
