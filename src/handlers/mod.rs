//! HTTP handlers module.
//!
//! Contains all route handlers organized by domain.

pub mod auth;
pub mod health;
pub mod agents;
pub mod credentials;
pub mod tokens;
pub mod stats;

use actix_web::web;

/// Configure all application routes.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Health check endpoints (no prefix)
    cfg.service(
        web::scope("/health")
            .service(health::liveness)
            .service(health::readiness)
    );

    // API v1 endpoints
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/health")
                    .service(health::liveness)
                    .service(health::readiness)
                    .service(health::detailed)
            )
            .configure(credentials::config)
            .service(
                web::scope("/agents")
                    .configure(agents::configure)
            )
            .configure(tokens::config)
            .configure(auth::configure)
            .configure(stats::config)
    );


}
