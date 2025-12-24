use actix_http::Request;
use actix_web::{test, web, App};
use sqlx::{Pool, Postgres};

// Import project modules. Assumes lib.rs exports these.
// We need to use "agentkey_backend" if that's the lib name in Cargo.toml
// or "crate" if we are in 'tests/' directory which sees lib as "agentkey_backend".
use agentkey_backend::{
    config::Config,
    handlers,
    models::{
        Agent, AgentResponse, AuthResponse, CreateAgentRequest, CreateCredentialRequest,
        CredentialResponse, DecryptedCredentialResponse, RegisterRequest, Team,
    },
    server::AppState,
    services::{
        agent::AgentService, auth::AuthService, credential::CredentialService, encryption::EncryptionService,
        jwt::JwtService,
    },
    utils::api_key::ApiKeyGenerator,
};
use std::sync::Arc;
use uuid::Uuid;

// Test helpers (setup_db, etc.) would be shared or copied.
// Since I don't see a shared "tests/common.rs", I'll replicate minimal setup.

async fn setup_app() -> (
    impl actix_web::dev::Service<Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    Pool<Postgres>,
    Config,
) {
    // Load config
    dotenvy::from_filename(".env.test").ok();
    let config = Config::from_env().expect("Failed to load config");

    // Db pool
    let pool = Pool::<Postgres>::connect(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Initialize Services
    let jwt_service = Arc::new(JwtService::new(
        config.jwt_secret.clone(),
        config.jwt_expiry_hours,
    ));
    let encryption_service = Arc::new(EncryptionService::new(config.encryption_key.clone())
        .expect("Failed to init encryption"));
    let credential_service = web::Data::new(CredentialService::new(encryption_service.clone()));
    let agent_service = web::Data::new(AgentService::new(jwt_service.clone()));
    let db_pool_data = web::Data::new(pool.clone());
    let jwt_service_data = web::Data::new(jwt_service.clone());

    let state = web::Data::new(AppState {
        db: agentkey_backend::db::Database::new(&config.database_url).await.unwrap(),
        redis: redis::Client::open("redis://127.0.0.1/").unwrap().get_connection_manager().await.unwrap(), 
        config: config.clone(),
    });

    // NOTE: Testing `handlers::configure_routes` requires properly configured App.
    // server.rs does:
    /*
        App::new()
            .app_data(state.clone())
            .app_data(db_pool.clone())
            .app_data(agent_service.clone())
            .app_data(credential_service.clone())
            .app_data(jwt_service_data.clone())
            .configure(handlers::configure_routes)
    */

    let app = test::init_service(
        App::new()
            .app_data(db_pool_data)
            .app_data(agent_service)
            .app_data(credential_service)
            .app_data(jwt_service_data)
            .configure(handlers::configure_routes),
    )
    .await;

    (app, pool, config)
}

#[actix_web::test]
async fn test_credential_lifecycle() {
    let (app, pool, _) = setup_app().await;

    // 1. Setup User & Team (via AuthService or raw SQL)
    // To cleanly test handlers, we should likely just insert data or use `RegisterRequest` handler if available.
    // Using Services is faster/cleaner than raw SQL. But I need `AuthService` instance which I didn't init in setup_app but I can.
    // Or just make HTTP request to /auth/register.
    
    // Register
    let register_req = RegisterRequest {
        email: format!("test_cred_{}@example.com", Uuid::new_v4()),
        password: "StrongPass!123".to_string(),
        team_name: Some("Test Team".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_req)
        .to_request();
    let resp: AuthResponse = test::call_and_read_body_json(&app, req).await;
    let token = resp.token;
    let agent_token = token.clone(); // User token used to manage agent
    
    // 2. Create Agent
    let create_agent_req = CreateAgentRequest {
        name: "test-agent-cred".to_string(),
        description: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/agents")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_agent_req)
        .to_request();
        
    // Wait, CreateAgentResponse struct definition might be needed.
    // Assuming JSON structure.
    let agent_resp_val: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let agent_id = agent_resp_val["agent"]["id"].as_str().unwrap().to_string();
    let api_key = agent_resp_val["api_key"].as_str().unwrap().to_string();

    // 3. Create Credential (User Auth)
    let create_cred_req = CreateCredentialRequest {
        name: "db_password".to_string(),
        credential_type: "password".to_string(),
        description: Some("Database password".to_string()),
        secret: "super_secret_value".to_string(),
        rotation_enabled: Some(true),
        rotation_interval_days: Some(30),
    };
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/agents/{}/credentials", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_cred_req)
        .to_request();
        
    let cred_resp: CredentialResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(cred_resp.name, "db_password");
    let credential_id = cred_resp.id;

    // 4. List Credentials (User Auth)
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}/credentials", agent_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let list_resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let items = list_resp["data"].as_array().unwrap();
    assert!(items.len() >= 1);

    // 5. Decrypt Credential (Agent Auth)
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}/credentials/{}/decrypt", agent_id, credential_id))
        .insert_header(("X-API-Key", api_key.clone()))
        .to_request();
    
    let decrypt_resp: DecryptedCredentialResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(decrypt_resp.secret, "super_secret_value");

    // 6. Access Control: Cross-Agent Access Check (Fail)
    // Create another agent
    let create_agent2_req = CreateAgentRequest { name: "agent-2".to_string(), description: None };
    let req = test::TestRequest::post().uri("/api/v1/agents").insert_header(("Authorization", format!("Bearer {}", token))).set_json(&create_agent2_req).to_request();
    let agent2_val: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let api_key2 = agent2_val["api_key"].as_str().unwrap();

    // Try to decrypt cred1 with agent2 key
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/agents/{}/credentials/{}/decrypt", agent_id, credential_id))
        .insert_header(("X-API-Key", api_key2))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Should fail: Path agent_id != Auth agent_id
    assert_eq!(resp.status(), actix_web::http::StatusCode::FORBIDDEN);

    // 7. Cleanup (Optional, test DB logic)
}
