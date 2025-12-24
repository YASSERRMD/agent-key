//! Data models for AgentKey.
//!
//! Contains database models, DTOs, and database access methods.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;
use validator::Validate;

use crate::errors::ApiError;
use crate::utils::api_key::ApiKeyGenerator;

// =============================================================================
// DATABASE MODELS
// =============================================================================

/// Team model representing a multi-tenant organization.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub plan: String,
    pub max_agents: i32,
    pub max_credentials: i32,
    pub max_monthly_reads: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// User model for authentication.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub team_id: Uuid,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Agent model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,  // active, suspended, archived
    pub api_key_hash: String,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Agent quota model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentQuota {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub team_id: Uuid,
    pub month_year: String, // e.g., "2025-01"
    pub api_calls_used: i32,
    pub api_calls_limit: i32,
    pub key_rotations_used: i32,
    pub key_rotations_limit: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent usage model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentUsage {
    pub id: i64,
    pub agent_id: Uuid,
    pub team_id: Uuid,
    pub event_type: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
}

// =============================================================================
// REQUEST/RESPONSE DTOs
// =============================================================================

/// User registration request.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 12, message = "Password must be at least 12 characters"))]
    pub password: String,
    
    #[validate(length(min = 1, max = 255, message = "Team name must be 1-255 characters"))]
    pub team_name: Option<String>,
}

/// User login request.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Authentication response with tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub email: String,
    pub role: String,
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Token refresh request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Token refresh response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub token: String,
    pub expires_in: i64,
}

/// User profile (without sensitive data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub team_id: Uuid,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request DTO for creating an agent.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateAgentRequest {
    #[validate(length(min = 3, max = 255, message = "Name must be 3-255 characters"))]
    #[validate(regex(path = "REGEX_ALPHANUM_HYPHEN", message = "Name must contain only alphanumeric characters and hyphens"))]
    pub name: String,
    
    pub description: Option<String>,
}

lazy_static::lazy_static! {
    static ref REGEX_ALPHANUM_HYPHEN: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9\-]+$").unwrap();
}

/// Response DTO for agent creation (includes API key).
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAgentResponse {
    pub agent: AgentResponse,
    pub api_key: String,
    pub warning: String,
}

/// Response DTO for agent (excludes API key).
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Request DTO for updating an agent.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateAgentRequest {
    #[validate(length(min = 3, max = 255))]
    #[validate(regex(path = "REGEX_ALPHANUM_HYPHEN"))]
    pub name: Option<String>,
    
    pub description: Option<String>,
    
    #[validate(custom(function = "validate_agent_status"))]
    pub status: Option<String>,
}

fn validate_agent_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "active" | "suspended" | "archived" => Ok(()),
        _ => Err(validator::ValidationError::new("Invalid status")),
    }
}

/// Pagination query parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
    pub pages: i32,
}

/// Response DTO for quota usage stats.
#[derive(Debug, Serialize)]
pub struct QuotaUsage {
    pub month: String,
    pub api_calls: QuotaMetric,
    pub key_rotations: QuotaMetric,
}

#[derive(Debug, Serialize)]
pub struct QuotaMetric {
    pub used: i32,
    pub limit: i32,
    pub percentage: f32,
}

// =============================================================================
// TEAM DATABASE METHODS
// =============================================================================

impl Team {
    /// Create a new team.
    pub async fn create(
        pool: &PgPool,
        name: &str,
        owner_id: Uuid,
        plan: &str,
    ) -> Result<Team, ApiError> {
        let team = sqlx::query_as::<_, Team>(
            r#"
            INSERT INTO teams (name, owner_id, plan)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .bind(plan)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(team)
    }

    /// Find a team by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Team>, ApiError> {
        let team = sqlx::query_as::<_, Team>(
            r#"
            SELECT * FROM teams 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(team)
    }

    /// Find a team by name.
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Team>, ApiError> {
        let team = sqlx::query_as::<_, Team>(
            r#"
            SELECT * FROM teams 
            WHERE LOWER(name) = LOWER($1) AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(team)
    }

    /// Update team owner.
    pub async fn update_owner(pool: &PgPool, id: Uuid, owner_id: Uuid) -> Result<Team, ApiError> {
        let team = sqlx::query_as::<_, Team>(
            r#"
            UPDATE teams 
            SET owner_id = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(owner_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(team)
    }

    /// Check if team has reached agent quota.
    pub async fn check_agent_quota(&self, pool: &PgPool) -> Result<bool, ApiError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM agents 
            WHERE team_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(self.id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(count.0 < self.max_agents as i64)
    }

    /// Check if team has reached credential quota.
    pub async fn check_credential_quota(&self, pool: &PgPool) -> Result<bool, ApiError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM credentials c
            JOIN agents a ON c.agent_id = a.id
            WHERE a.team_id = $1 AND c.deleted_at IS NULL
            "#,
        )
        .bind(self.id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(count.0 < self.max_credentials as i64)
    }

    /// Upgrade team plan.
    pub async fn upgrade_plan(
        pool: &PgPool,
        id: Uuid,
        plan: &str,
        max_agents: i32,
        max_credentials: i32,
        max_monthly_reads: i32,
    ) -> Result<Team, ApiError> {
        let team = sqlx::query_as::<_, Team>(
            r#"
            UPDATE teams 
            SET plan = $2, max_agents = $3, max_credentials = $4, 
                max_monthly_reads = $5, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(plan)
        .bind(max_agents)
        .bind(max_credentials)
        .bind(max_monthly_reads)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(team)
    }
}

// =============================================================================
// USER DATABASE METHODS
// =============================================================================

impl User {
    /// Create a new user.
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
        team_id: Uuid,
        role: &str,
    ) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, team_id, role)
            VALUES (LOWER($1), $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(team_id)
        .bind(role)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
                ApiError::Conflict("Email already registered".to_string())
            } else {
                ApiError::DatabaseError(e.to_string())
            }
        })?;

        Ok(user)
    }

    /// Find a user by email (case-insensitive).
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE LOWER(email) = LOWER($1) AND deleted_at IS NULL
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Find a user by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Find all users in a team.
    pub async fn find_by_team(pool: &PgPool, team_id: Uuid) -> Result<Vec<User>, ApiError> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE team_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(team_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(users)
    }

    /// Update user's last login timestamp.
    pub async fn update_last_login(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE users 
            SET last_login = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Soft delete a user.
    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE users 
            SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Update user's role.
    pub async fn update_role(pool: &PgPool, id: Uuid, role: &str) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET role = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(role)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Convert user to public profile (without password hash).
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            email: self.email.clone(),
            team_id: self.team_id,
            role: self.role.clone(),
            is_active: self.is_active,
            last_login: self.last_login,
            created_at: self.created_at,
        }
    }
}

// =============================================================================
// AGENT DATABASE METHODS
// =============================================================================

impl Agent {
    /// Create a new agent.
    pub async fn create(
        pool: &PgPool,
        team_id: Uuid,
        name: &str,
        description: Option<String>,
        created_by: Uuid,
    ) -> Result<(Agent, String), ApiError> {
        // Generate and hash API key
        let api_key = ApiKeyGenerator::generate();
        let api_key_hash = ApiKeyGenerator::hash(&api_key);

        let mut tx = pool.begin().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // Insert agent
        let agent = sqlx::query_as::<_, Agent>(
            r#"
            INSERT INTO agents (team_id, name, description, api_key_hash, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(team_id)
        .bind(name)
        .bind(description)
        .bind(&api_key_hash)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
                ApiError::Conflict("Agent name already exists in team".to_string())
            } else {
                ApiError::DatabaseError(e.to_string())
            }
        })?;

        // Insert initial API key record
        sqlx::query(
            r#"
            INSERT INTO agent_api_keys (agent_id, api_key_hash)
            VALUES ($1, $2)
            "#,
        )
        .bind(agent.id)
        .bind(&api_key_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        tx.commit().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok((agent, api_key))
    }

    /// Find an agent by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Agent>, ApiError> {
        let agent = sqlx::query_as::<_, Agent>(
            r#"
            SELECT * FROM agents 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(agent)
    }

    /// Find an agent by API key hash.
    pub async fn find_by_api_key_hash(pool: &PgPool, key_hash: &str) -> Result<Option<Agent>, ApiError> {
        // First check if key is active in agent_api_keys
        let api_key_record = sqlx::query(
            r#"
            SELECT agent_id FROM agent_api_keys 
            WHERE api_key_hash = $1 AND status = 'active'
            "#,
        )
        .bind(key_hash)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if let Some(record) = api_key_record {
            let agent_id: Uuid = record.get("agent_id");
            
            // Get the agent
            let agent = sqlx::query_as::<_, Agent>(
                r#"
                SELECT * FROM agents 
                WHERE id = $1 AND deleted_at IS NULL AND status = 'active'
                "#,
            )
            .bind(agent_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            Ok(agent)
        } else {
            Ok(None)
        }
    }

    /// Find agents by team (paginated).
    pub async fn find_by_team(
        pool: &PgPool,
        team_id: Uuid,
        page: i32,
        limit: i32,
    ) -> Result<(Vec<Agent>, i64), ApiError> {
        let offset = (page - 1) * limit;

        let agents = sqlx::query_as::<_, Agent>(
            r#"
            SELECT * FROM agents 
            WHERE team_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(team_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM agents 
            WHERE team_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok((agents, count.0))
    }

    /// Update agent.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        status: Option<String>,
    ) -> Result<Agent, ApiError> {
        let mut query = "UPDATE agents SET updated_at = CURRENT_TIMESTAMP".to_string();
        let mut params_count = 1; // start after id ($1)

        // Using a transaction to ensure no partial updates
        let mut tx = pool.begin().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        // This is a bit simplified; for a dynamic query builder we might want more logic
        // But for 3 optional fields, we can just check them one by one
        
        if name.is_some() {
            params_count += 1;
            query.push_str(&format!(", name = ${}", params_count));
        }
        if description.is_some() {
            params_count += 1;
            query.push_str(&format!(", description = ${}", params_count));
        }
        if status.is_some() {
            params_count += 1;
            query.push_str(&format!(", status = ${}", params_count));
        }

        query.push_str(" WHERE id = $1 AND deleted_at IS NULL RETURNING *");

        let mut q = sqlx::query_as::<_, Agent>(&query).bind(id);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(d) = description {
            q = q.bind(d);
        }
        if let Some(s) = status {
            q = q.bind(s);
        }

        let agent = q
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
                    ApiError::Conflict("Agent name already exists in team".to_string())
                } else {
                    ApiError::DatabaseError(e.to_string())
                }
            })?;

        tx.commit().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(agent)
    }

    /// Soft delete agent.
    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
        let mut tx = pool.begin().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // Soft delete agent
        sqlx::query(
            r#"
            UPDATE agents 
            SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // Revoke all API keys
        sqlx::query(
            r#"
            UPDATE agent_api_keys 
            SET status = 'revoked', revoked_at = CURRENT_TIMESTAMP
            WHERE agent_id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        tx.commit().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Count active agents in team.
    pub async fn count_by_team(pool: &PgPool, team_id: Uuid) -> Result<i64, ApiError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM agents 
            WHERE team_id = $1 AND deleted_at IS NULL AND status = 'active'
            "#,
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(count.0)
    }

    /// Update usage timestamp and count.
    pub async fn update_last_used(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE agents 
            SET last_used = CURRENT_TIMESTAMP, 
                usage_count = usage_count + 1,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Convert to public response (API key omitted).
    pub fn to_response(&self) -> AgentResponse {
        AgentResponse {
            id: self.id,
            team_id: self.team_id,
            name: self.name.clone(),
            description: self.description.clone(),
            status: self.status.clone(),
            last_used: self.last_used,
            usage_count: self.usage_count,
            created_at: self.created_at,
        }
    }
}

// =============================================================================
// AUDIT EVENT LOGGING
// =============================================================================

/// Log an audit event.
pub async fn log_audit_event(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Option<Uuid>,
    event_type: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    change_description: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events 
            (team_id, user_id, event_type, resource_type, resource_id, change_description, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6, $7::inet)
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .bind(event_type)
    .bind(resource_type)
    .bind(resource_id)
    .bind(change_description)
    .bind(ip_address)
    .execute(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_to_profile() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed".to_string(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
            is_active: true,
            last_login: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let profile = user.to_profile();
        
        assert_eq!(profile.id, user.id);
        assert_eq!(profile.email, user.email);
        assert_eq!(profile.role, user.role);
        // Password hash should not be in profile
    }

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: Some("My Team".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = RegisterRequest {
            email: "not-an-email".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: None,
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = LoginRequest {
            email: "invalid".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_create_agent_request_validation() {
        let valid = CreateAgentRequest {
            name: "my-agent-123".to_string(),
            description: None,
        };
        assert!(valid.validate().is_ok());

        let invalid_chars = CreateAgentRequest {
            name: "agent with space".to_string(),
            description: None,
        };
        assert!(invalid_chars.validate().is_err());

        let too_short = CreateAgentRequest {
            name: "ab".to_string(),
            description: None,
        };
        assert!(too_short.validate().is_err());
    }
}
