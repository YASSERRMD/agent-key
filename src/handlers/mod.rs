//! HTTP handlers module.
//!
//! Contains all route handlers organized by domain.

pub mod health;

use actix_web::web;

/// Configure all application routes.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check endpoints (no prefix)
        .service(
            web::scope("/health")
                .service(health::liveness)
                .service(health::readiness)
        )
        // API v1 endpoints
        .service(
            web::scope("/api/v1")
                .service(
                    web::scope("/health")
                        .service(health::liveness)
                        .service(health::readiness)
                        .service(health::detailed)
                )
        );
}
