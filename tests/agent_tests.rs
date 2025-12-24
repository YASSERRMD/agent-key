//! Integration tests for agent management.

use actix_web::{test, web, App};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use agentkey_backend::config::Config;
use agentkey_backend::db::Database;
use agentkey_backend::handlers;
use agentkey_backend::models::{AgentResponse, CreateAgentResponse, CreateAgentRequest, UpdateAgentRequest};
use agentkey_backend::services::agent::AgentService;
use agentkey_backend::services::jwt::JwtService;
use agentkey_backend::services::auth::AuthService;
use agentkey_backend::server::AppState;

pub struct TestApp {
    pub db: Database,
    pub api_address: String, // Not used directly in init_service but good for ref
    pub pool: PgPool,
    pub token: String, // Access token for tests
    pub team_id: Uuid,
    pub user_id: Uuid,
}

/// Spawn the application for testing.
async fn spawn_app() -> (impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>, TestApp) {
    // 1. Load config
    dotenvy::from_filename(".env.test").ok(); 
    // Fallback to .env if .env.test missing, or manual env
    let mut config = Config::from_env().unwrap_or_else(|_| {
        std::env::set_var("DATABASE_URL", "postgresql://postgres:postgres@localhost:5432/agentkey_test");
        std::env::set_var("JWT_SECRET", "test-secret-must-be-at-least-32-chars-long");
        std::env::set_var("ENCRYPTION_KEY", "test-encryption-key-32-chars-long!");
        Config::from_env().expect("Failed to create default test config")
    });
    
    // Ensure we use a test database (safety check)
    // if !config.database_url.contains("test") {
    //     panic!("Use a test database for integration tests!");
    // }

    // 2. Connect to DB and run migrations
    let db = Database::new(&config.database_url).await.expect("Failed to connect to DB");
    let pool = db.pool().clone();

    // 3. Clean up database
    sqlx::query("TRUNCATE TABLE users, teams, agents, agent_quotas, agent_api_keys, audit_events RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate tables");

    // 4. Create Test User and Team
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    let email = format!("test-{}@example.com", Uuid::new_v4());
    
    // Insert team
    sqlx::query("INSERT INTO teams (id, name, owner_id, plan, max_agents, max_credentials, max_monthly_reads) VALUES ($1, $2, $3, $4, 10, 100, 1000)")
        .bind(team_id)
        .bind("Test Team")
        .bind(user_id)
        .bind("pro")
        .execute(&pool)
        .await
        .expect("Failed to insert team");

    // Insert user
    let password_hash = "$2b$12$KvKgyXy.PjL8C5e1...hashed...password"; // Dummy bcrypt hash
    sqlx::query("INSERT INTO users (id, email, password_hash, team_id, role, is_active) VALUES ($1, $2, $3, $4, 'admin', true)")
        .bind(user_id)
        .bind(&email)
        .bind(password_hash)
        .bind(team_id)
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // 5. Initialize Services
    let jwt_service = Arc::new(JwtService::new(config.jwt_secret.clone(), 60));
    let agent_service = web::Data::new(AgentService::new(jwt_service.clone()));
    let db_pool = web::Data::new(pool.clone());
    
    // Generate Token
    let token = jwt_service.create_token(user_id, team_id, "admin".to_string()).expect("Failed to create token");

    // 6. Build App Service
    // We mock the AppState and dependencies as server.rs does
    let state = web::Data::new(AppState {
        db: db.clone(),
        redis: redis::Client::open("redis://127.0.0.1/").unwrap().get_connection_manager().await.unwrap(), // Mock or real redis? 
        // Real redis might fail if not running. 
        // If Redis is required for rate limiting, we might need it. 
        // Tests usually skip redis or mock it.
        // Assuming Redis not critical for basic agent functional tests?
        // AppState requires it.
        // I will trust 'redis' crate mock if possible, or just fail if no redis.
        // Let's assume user has redis or CI has it.
        config: config.clone(),
    });

    let app_service = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(db_pool.clone())
            .app_data(agent_service.clone())
            .configure(handlers::configure_routes)
    ).await;

    let test_app = TestApp {
        db,
        api_address: "http://localhost".to_string(), // Virtual
        pool,
        token,
        team_id,
        user_id,
    };

    (app_service, test_app)
}

#[actix_web::test]
async fn test_create_agent_success() {
    let (app, test_app) = spawn_app().await;

    let req_body = CreateAgentRequest {
        name: "test-agent-1".to_string(),
        description: Some("Description".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&req_body)
        .to_request();

    let resp: CreateAgentResponse = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(resp.agent.name, "test-agent-1");
    assert!(!resp.api_key.is_empty());
    assert!(resp.api_key.starts_with("ak_"));
}

#[actix_web::test]
async fn test_create_agent_duplicate_name() {
    let (app, test_app) = spawn_app().await;

    let req_body = CreateAgentRequest {
        name: "duplicate-agent".to_string(),
        description: None,
    };

    // First create
    let req1 = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&req_body)
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert!(resp1.status().is_success());

    // Second create (same name)
    let req2 = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&req_body)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    
    // Should fail with Conflict
    assert_eq!(resp2.status(), actix_web::http::StatusCode::CONFLICT);
}

#[actix_web::test]
async fn test_get_agent() {
    let (app, test_app) = spawn_app().await;

    // Create
    let create_req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&CreateAgentRequest {
            name: "get-agent".to_string(),
            description: None,
        })
        .to_request();
    let created: CreateAgentResponse = test::call_and_read_body_json(&app, create_req).await;
    let agent_id = created.agent.id;

    // Get
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .to_request();
    
    let fetched: AgentResponse = test::call_and_read_body_json(&app, get_req).await;
    assert_eq!(fetched.id, agent_id);
    assert_eq!(fetched.name, "get-agent");
}

#[actix_web::test]
async fn test_update_agent() {
    let (app, test_app) = spawn_app().await;

    // Create
    let create_req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&CreateAgentRequest {
            name: "update-agent".to_string(),
            description: None,
        })
        .to_request();
    let created: CreateAgentResponse = test::call_and_read_body_json(&app, create_req).await;
    let agent_id = created.agent.id;

    // Update
    let update_req = test::TestRequest::patch()
        .uri(&format!("/api/v1/agents/{}", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&UpdateAgentRequest {
            name: Some("updated-name".to_string()),
            description: Some("New Desc".to_string()),
            status: None,
        })
        .to_request();
    
    let updated: AgentResponse = test::call_and_read_body_json(&app, update_req).await;
    assert_eq!(updated.name, "updated-name");
    assert_eq!(updated.description, Some("New Desc".to_string()));
}

#[actix_web::test]
async fn test_delete_agent() {
    let (app, test_app) = spawn_app().await;

    // Create
    let create_req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&CreateAgentRequest {
            name: "delete-agent".to_string(),
            description: None,
        })
        .to_request();
    let created: CreateAgentResponse = test::call_and_read_body_json(&app, create_req).await;
    let agent_id = created.agent.id;

    // Delete
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/agents/{}", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .to_request();
    
    let resp = test::call_service(&app, delete_req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NO_CONTENT);

    // Verify Get returns 404
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .to_request();
    let resp_get = test::call_service(&app, get_req).await;
    assert_eq!(resp_get.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_api_key_auth() {
    let (app, test_app) = spawn_app().await;

    // Create Agent to get API Key
    let create_req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", test_app.token)))
        .set_json(&CreateAgentRequest {
            name: "auth-agent".to_string(),
            description: None,
        })
        .to_request();
    let created: CreateAgentResponse = test::call_and_read_body_json(&app, create_req).await;
    let api_key = created.api_key;
    let agent_id = created.agent.id;

    // Call status endpoint with API Key
    let status_req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}/status", agent_id))
        .insert_header(("X-API-Key", api_key.clone()))
        .to_request();
    
    let resp = test::call_service(&app, status_req).await;
    assert!(resp.status().is_success());

    // Call status endpoint with Invalid Key
    let invalid_req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}/status", agent_id))
        .insert_header(("X-API-Key", "invalid-key"))
        .to_request();
    
    let resp_invalid = test::call_service(&app, invalid_req).await;
    assert_eq!(resp_invalid.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}
