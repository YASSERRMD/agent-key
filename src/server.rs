//! Actix-web server configuration and startup.
//!
//! Configures middleware, routes, and application state.

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use redis::aio::ConnectionManager;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::config::Config;
use crate::db::Database;
use crate::handlers;

/// Application state shared across all handlers
pub struct AppState {
    pub db: Database,
    pub redis: ConnectionManager,
    pub config: Config,
}

/// Run the HTTP server with the given configuration.
///
/// # Arguments
///
/// * `addr` - Server bind address (e.g., "127.0.0.1:8080")
/// * `db` - Database connection pool
/// * `redis` - Redis connection manager
/// * `config` - Application configuration
///
/// # Errors
///
/// Returns `std::io::Error` if the server fails to start.
pub async fn run(
    addr: String,
    db: Database,
    redis: ConnectionManager,
    config: Config,
) -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        db,
        redis,
        config: config.clone(),
    });

    info!("Configuring HTTP server...");

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin() // TODO: Restrict in production
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            // Middleware
            .wrap(TracingLogger::default())
            .wrap(cors)
            // Routes
            .configure(handlers::configure_routes)
    })
    .bind(&addr)?
    .workers(num_cpus::get().max(2))
    .shutdown_timeout(30)
    .run()
    .await
}

/// Get the number of CPUs available
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_cpus() {
        let cpus = num_cpus::get();
        assert!(cpus >= 1);
    }
}
