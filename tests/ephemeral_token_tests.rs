//! Integration tests for ephemeral token service and endpoints.

use actix_web::{test, web, App};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;
use chrono::{Utc, Duration};
use aes_gcm::aead::KeyInit; // For Aes256Gcm

use agentkey_backend::config::Config;
use agentkey_backend::models::{Agent, Credential, EphemeralToken, EphemeralTokenResponse, TokenStatus};
use agentkey_backend::services::encryption::EncryptionService;
use agentkey_backend::services::ephemeral_token::EphemeralTokenService;
use agentkey_backend::services::credential::CredentialService;
use agentkey_backend::handlers;

// Test Setup Helpers
async fn setup_db() -> PgPool {
    let config = Config::from_env().expect("Failed to load config");
    // Assume test DB is configured in .env.test or similar
    // For now, using the pool from main config (careful with data!)
    // Ideally use a test container or separate DB.
    // Given the environment constraints, we use the configured DB but cleanup.
    
    let pool = PgPool::connect(&config.database_url).await.expect("Failed to connect to DB");
    
    // Run migrations (idempotent)
    sqlx::migrate!("./migrations").run(&pool).await.expect("Failed migraions");
    
    pool
}

async fn create_test_agent(pool: &PgPool, name: &str) -> (Uuid, Uuid, Uuid) {
    // Requires User and Team first. 
    // This is elaborate. Let's reuse existing if checking logic is hard.
    // Or insert raw SQL for speed.
    
    let team_id = Uuid::new_v4();
    let owner_id = Uuid::new_v4();
    
    // Insert dummy user/team/agent
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash, role, is_active) VALUES ($1, $2, $3, $4, true)",
        owner_id, format!("owner_{}@test.com", Uuid::new_v4()), "hash", "admin"
    ).execute(pool).await.ok(); // Ignore if exists
    
    // Create actual team
    let team = sqlx::query!(
        "INSERT INTO teams (id, name, owner_id) VALUES ($1, $2, $3) RETURNING id",
        team_id, format!("Team {}", name), owner_id
    ).fetch_one(pool).await.unwrap();

    let agent_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO agents (id, team_id, name, created_by, status) VALUES ($1, $2, $3, $4, 'active')",
        agent_id, team.id, name, owner_id
    ).execute(pool).await.unwrap();

    (agent_id, team.id, owner_id)
}

async fn create_test_credential(
    pool: &PgPool,
    service: &CredentialService,
    agent_id: Uuid,
    team_id: Uuid,
    owner_id: Uuid,
    name: &str
) -> Credential {
    // Encrypt "secret-value"
    // Using service.create_credential would be cleaner but requires Request object.
    
    // Manually insert for speed/simplicity or use service.
    // Use service to ensure valid encryption.
    use agentkey_backend::models::CreateCredentialRequest;
    
    let req = CreateCredentialRequest {
        name: name.to_string(),
        value: "secret-value-123".to_string(),
        description: None,
        credential_type: "password".to_string(),
        metadata: None,
        tags: None,
        rotation_policy: None,
    };
    
    service.create_credential(pool, agent_id, team_id, owner_id, req).await.expect("Failed to create cred")
}

#[actix_web::test]
async fn test_generate_and_verify_token_flow() {
    let pool = setup_db().await;
    let config = Config::from_env().unwrap();
    
    let encryption = std::sync::Arc::new(EncryptionService::new(config.encryption_key.clone()).unwrap());
    let token_service = web::Data::new(EphemeralTokenService::new(config.jwt_secret.clone(), encryption.clone()));
    let credential_service = CredentialService::new(encryption.clone());

    // 1. Setup Data
    let (agent_id, team_id, owner_id) = create_test_agent(&pool, "token_test_agent").await;
    let cred = create_test_credential(&pool, &credential_service, agent_id, team_id, owner_id, "db_pass").await;

    // 2. Generate Token
    let response = token_service.generate_token(
        &pool,
        agent_id,
        "db_pass",
        Some("127.0.0.1")
    ).await.expect("Generate token failed");

    assert!(!response.token.is_empty());
    assert_eq!(response.credential_name, "db_pass");

    // 3. Verify Token
    let verified = token_service.verify_token(
        &pool,
        &response.token,
        Some("127.0.0.1")
    ).await.expect("Verify token failed");

    assert_eq!(verified.secret, "secret-value-123");
    assert_eq!(verified.agent_id, agent_id);
    assert_eq!(verified.credential_id, cred.id);

    // 4. Check Status
    let status = token_service.get_token_status(&pool, &verified.jti).await.unwrap();
    assert_eq!(status.status, "active");

    // 5. Revoke Token
    token_service.revoke_token(&pool, &verified.jti, Some("127.0.0.1")).await.unwrap();

    // 6. Verify Revoked status
    let status_revoked = token_service.get_token_status(&pool, &verified.jti).await.unwrap();
    assert_eq!(status_revoked.status, "revoked");

    // 7. Verify fails after revocation
    let verify_result = token_service.verify_token(&pool, &response.token, None).await;
    assert!(verify_result.is_err());
}

#[actix_web::test]
async fn test_token_expiration() {
    let pool = setup_db().await;
    let config = Config::from_env().unwrap();
    let encryption = std::sync::Arc::new(EncryptionService::new(config.encryption_key.clone()).unwrap());
    
    // Short TTL: 1 second
    let token_service = web::Data::new(EphemeralTokenService::with_ttl(config.jwt_secret.clone(), encryption.clone(), 1));
    let credential_service = CredentialService::new(encryption.clone());

    let (agent_id, team_id, owner_id) = create_test_agent(&pool, "expire_test_agent").await;
    let _cred = create_test_credential(&pool, &credential_service, agent_id, team_id, owner_id, "api_key").await;

    let response = token_service.generate_token(&pool, agent_id, "api_key", None).await.unwrap();

    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Verify should fail due to JWT expiration
    let result = token_service.verify_token(&pool, &response.token, None).await;
    assert!(result.is_err());
    
    // Check DB status (expired logic in cleanup)
    // Does cleanup run automatically? No.
    // But get_token_status checks time.
    
    // Need to extract JTI from token to check status, but parse logic is internal or in util?
    // We can rely on Verify failing.
}

#[actix_web::test]
async fn test_access_other_agents_credential_fails() {
    let pool = setup_db().await;
    let config = Config::from_env().unwrap();
    let encryption = std::sync::Arc::new(EncryptionService::new(config.encryption_key.clone()).unwrap());
    let token_service = web::Data::new(EphemeralTokenService::new(config.jwt_secret.clone(), encryption.clone()));
    let credential_service = CredentialService::new(encryption.clone());

    let (agent1, team1, owner1) = create_test_agent(&pool, "agent1").await;
    let (agent2, team2, owner2) = create_test_agent(&pool, "agent2").await;
    
    let _cred1 = create_test_credential(&pool, &credential_service, agent1, team1, owner1, "secret1").await;

    // Agent 2 tries to get Agent 1's credential token
    // Service check: `Credential::find_by_name(pool, agent_id, ...)`
    // If we pass agent1's ID but allow agent2 to call?
    // The handler checks permission. Service assumes caller logic.
    // But `Credential::find_by_name` filters by `agent_id`.
    // So if agent2 requests credential "secret1" (which belongs to agent1), it won't find it under agent2's credentials.
    
    let result = token_service.generate_token(&pool, agent2, "secret1", None).await;
    assert!(result.is_err()); // Not found because secret1 is not linked to agent2
}
