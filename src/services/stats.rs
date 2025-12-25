//! Statistics service for dashboard analytics.

use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::ApiError;
use crate::models::{DashboardStats, ActivityLog};

/// Service for aggregating dashboard statistics.
pub struct StatsService;

impl StatsService {
    /// Get dashboard statistics for a team.
    pub async fn get_team_stats(pool: &PgPool, team_id: Uuid) -> Result<DashboardStats, ApiError> {
        // 1. Total Agents
        let total_agents: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM agents WHERE team_id = $1 AND deleted_at IS NULL"
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // 2. Total Credentials
        let total_credentials: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM credentials WHERE team_id = $1 AND deleted_at IS NULL"
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // 3. API Access Count (Last 30 days)
        let api_access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_events 
             WHERE team_id = $1 
             AND event_type = 'credential.read' 
             AND created_at > NOW() - INTERVAL '30 days'"
        )
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

         // 4. Success Rate (Mocked for now as we don't log failures explicitly in audit_events yet)
         // In a real scenario, this would filter by status 'success' vs 'failure' if available.
        let success_rate = 99.9; 

        // 5. Recent Activity
        // We query audit_events and map them to ActivityLog.
        // Assuming audit_events has: id, event_type, resource_type, resource_id, created_at, ip_address (inet)
        // We'll construct a friendly description string.
        let recent_activity_rows = sqlx::query!(
            r#"
            SELECT id, event_type, resource_type, resource_id, created_at, ip_address::text as ip_address
            FROM audit_events
            WHERE team_id = $1
            ORDER BY created_at DESC
            LIMIT 5
            "#,
            team_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let recent_activity: Vec<ActivityLog> = recent_activity_rows.into_iter().map(|row| {
            let description = format!("{} on {}", row.event_type, row.resource_type.unwrap_or_default());
            ActivityLog {
                id: row.id,
                description,
                timestamp: row.created_at,
                status: "Success".to_string(), // Defaulting to Success as we log successful actions primarily
                ip_address: row.ip_address.map(|ip| serde_json::Value::String(ip)),
            }
        }).collect();

        Ok(DashboardStats {
            total_agents: total_agents.0,
            total_credentials: total_credentials.0,
            api_access_count: api_access_count.0,
            success_rate,
            recent_activity,
        })
    }
}
