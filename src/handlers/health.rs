//! Health check endpoints.
//!
//! Provides liveness and readiness probes for Kubernetes/Docker health checks.

use actix_web::{get, web, HttpResponse};
use chrono::Utc;
use serde::Serialize;
use tracing::warn;

use crate::server::AppState;

/// Simple health response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detailed health response with component statuses
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub environment: String,
    pub components: ComponentHealth,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Component health statuses
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub database: ComponentStatus,
    pub redis: ComponentStatus,
}

/// Individual component status
#[derive(Debug, Serialize)]
pub struct ComponentStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Liveness probe endpoint.
///
/// Returns 200 OK if the service is running. This is a simple check
/// that doesn't verify external dependencies.
///
/// # Route
///
/// `GET /health` or `GET /health/live`
#[get("")]
pub async fn liveness() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
    })
}

/// Readiness probe endpoint.
///
/// Returns 200 OK only if all dependencies (database, redis) are healthy.
/// Returns 503 Service Unavailable if any dependency is unhealthy.
///
/// # Route
///
/// `GET /health/ready`
#[get("/ready")]
pub async fn readiness(state: web::Data<AppState>) -> HttpResponse {
    let db_healthy = check_database(&state).await;
    let redis_healthy = check_redis(&state).await;

    let is_ready = db_healthy && redis_healthy;

    let response = HealthResponse {
        status: if is_ready { "ready".to_string() } else { "not_ready".to_string() },
        timestamp: Utc::now(),
    };

    if is_ready {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// Detailed health check endpoint.
///
/// Returns comprehensive health information including:
/// - Overall status
/// - Individual component statuses with latency
/// - Version and environment info
///
/// # Route
///
/// `GET /health/detailed` or `GET /api/v1/health/detailed`
#[get("/detailed")]
pub async fn detailed(state: web::Data<AppState>) -> HttpResponse {
    // Check database health with timing
    let db_start = std::time::Instant::now();
    let db_result = state.db.health_check().await;
    let db_latency = db_start.elapsed().as_millis() as u64;

    let db_status = match db_result {
        Ok(_) => ComponentStatus {
            status: "healthy".to_string(),
            latency_ms: Some(db_latency),
            error: None,
        },
        Err(e) => {
            warn!(error = %e, "Database health check failed");
            ComponentStatus {
                status: "unhealthy".to_string(),
                latency_ms: Some(db_latency),
                error: Some(e.to_string()),
            }
        }
    };

    // Check Redis health with timing
    let redis_start = std::time::Instant::now();
    let redis_result = check_redis_with_ping(&state).await;
    let redis_latency = redis_start.elapsed().as_millis() as u64;

    let redis_status = match redis_result {
        Ok(_) => ComponentStatus {
            status: "healthy".to_string(),
            latency_ms: Some(redis_latency),
            error: None,
        },
        Err(e) => {
            warn!(error = %e, "Redis health check failed");
            ComponentStatus {
                status: "unhealthy".to_string(),
                latency_ms: Some(redis_latency),
                error: Some(e),
            }
        }
    };

    // Determine overall status
    let overall_status = if db_status.status == "healthy" && redis_status.status == "healthy" {
        "healthy"
    } else if db_status.status == "healthy" || redis_status.status == "healthy" {
        "degraded"
    } else {
        "unhealthy"
    };

    let response = DetailedHealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: state.config.environment.clone(),
        components: ComponentHealth {
            database: db_status,
            redis: redis_status,
        },
        timestamp: Utc::now(),
    };

    match overall_status {
        "healthy" => HttpResponse::Ok().json(response),
        "degraded" => HttpResponse::Ok().json(response),
        _ => HttpResponse::ServiceUnavailable().json(response),
    }
}

/// Check database connectivity
async fn check_database(state: &web::Data<AppState>) -> bool {
    state.db.health_check().await.is_ok()
}

/// Check Redis connectivity
async fn check_redis(state: &web::Data<AppState>) -> bool {
    check_redis_with_ping(state).await.is_ok()
}

/// Check Redis connectivity with PING command
async fn check_redis_with_ping(state: &web::Data<AppState>) -> Result<(), String> {
    let mut conn = state.redis.clone();
    let result: Result<String, _> = redis::cmd("PING")
        .query_async(&mut conn)
        .await;

    match result {
        Ok(response) if response == "PONG" => Ok(()),
        Ok(response) => Err(format!("Unexpected response: {}", response)),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "ok".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_component_status_serialization() {
        let status = ComponentStatus {
            status: "healthy".to_string(),
            latency_ms: Some(5),
            error: None,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"latency_ms\":5"));
        assert!(!json.contains("error")); // None should be skipped
    }
}
