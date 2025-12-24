//! AgentKey Backend Server
//!
//! Entry point for the AgentKey credential management platform.

use agentkey_backend::{config::Config, db::Database, server};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber for structured logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,agentkey_backend=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting AgentKey backend server...");

    // Load configuration from environment
    let config = Config::from_env().expect("Failed to load configuration");

    info!(
        environment = %config.environment,
        server = %format!("{}:{}", config.server_host, config.server_port),
        "Configuration loaded"
    );

    // Initialize database connection pool
    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    info!("Database connection pool established");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Failed to create Redis client");

    let redis_conn = redis_client
        .get_connection_manager()
        .await
        .expect("Failed to connect to Redis");

    info!("Redis connection established");

    // Start HTTP server
    let server_addr = format!("{}:{}", config.server_host, config.server_port);
    info!(address = %server_addr, "Starting HTTP server");

    server::run(server_addr, db, redis_conn, config).await
}
