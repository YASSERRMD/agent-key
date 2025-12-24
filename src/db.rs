//! Database module for AgentKey backend.
//!
//! Handles PostgreSQL connection pooling and migrations.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use thiserror::Error;
use tracing::info;

/// Database-related errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Query error: {0}")]
    QueryError(#[from] sqlx::Error),
}

/// Database wrapper providing connection pool and utilities.
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool.
    ///
    /// This will also run any pending migrations.
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection URL
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError` if connection or migration fails.
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        info!("Connecting to PostgreSQL database...");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        info!("Database connection pool established");

        // Run migrations
        info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

        info!("Database migrations completed successfully");

        Ok(Database { pool })
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Perform a health check on the database connection.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if healthy, `Err` otherwise.
    pub async fn health_check(&self) -> Result<bool, DatabaseError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(true)
    }

    /// Get the current connection pool statistics.
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle_connections: self.pool.num_idle(),
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Current pool size
    pub size: u32,
    /// Number of idle connections
    pub idle_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_stats_creation() {
        let stats = PoolStats {
            size: 10,
            idle_connections: 5,
        };
        assert_eq!(stats.size, 10);
        assert_eq!(stats.idle_connections, 5);
    }
}
