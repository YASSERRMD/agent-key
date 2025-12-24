//! API Key authentication middleware.
//!
//! Extracts and validates API keys from the `X-API-Key` header.
//! Used for agent-to-system communication.

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures::future::ready;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::services::agent::AgentService;

/// Authenticated agent identity.
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub agent_id: Uuid,
    pub team_id: Uuid,
}

impl FromRequest for ApiKeyAuth {
    type Error = ApiError;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // 1. Extract API key
        let api_key = match req.headers().get("X-API-Key") {
            Some(k) => match k.to_str() {
                Ok(s) => s,
                Err(_) => return Box::pin(ready(Err(ApiError::Unauthorized("Invalid API key header".to_string())))),
            },
            None => return Box::pin(ready(Err(ApiError::Unauthorized("Missing API key header".to_string())))),
        };

        // 2. Get dependencies
        let pool = match req.app_data::<web::Data<PgPool>>() {
            Some(p) => p,
            None => return Box::pin(ready(Err(ApiError::InternalError("Database pool not found".to_string())))),
        };

        let service = match req.app_data::<web::Data<AgentService>>() {
            Some(s) => s,
            None => return Box::pin(ready(Err(ApiError::InternalError("Agent service not found".to_string())))),
        };

        // 3. Verify key (Synchronous blocking wait is not ideal in from_request, but Acceptable for simple auth
        // unless we use `std::future` logic which `from_request` allows but `Ready` return type implies synchronous or immediate.
        // Actually, FromRequest returns a Future. If I return `Ready`, it's immediate.
        // But database access is async `verify_api_key`.
        // So I CANNOT use `Ready`. I must return `Pin<Box<dyn Future...>>`.
        
        // Let's change Future type.
        // impl FromRequest for ApiKeyAuth ...
        // type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;
        
        let pool = pool.get_ref().clone();
        let service = service.clone(); // Clone the web::Data wrapper (Arc)
        // Wait, AgentService is `Clone`? No.
        // `web::Data<T>` is a wrapper around `Arc<T>`. 
        // Calling `clone()` on `web::Data` increments ref count.
        // `AgentService` itself doesn't implement Clone unless I derive it.
        // But `web::Data` is cheap to clone.
        // BUT `req.app_data` returns `&web::Data<T>`.
        // I can clone the `Data` wrapper.
        // `let service = service.clone();`
        
        let api_key_owned = api_key.to_string();

        Box::pin(async move {
            let (agent_id, team_id) = service.verify_api_key(&pool, &api_key_owned).await?;
            Ok(ApiKeyAuth { agent_id, team_id })
        })
    }
}
