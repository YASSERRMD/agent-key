use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;
use crate::services::stats::StatsService;

/// Get dashboard statistics for the authenticated user's team.
pub async fn get_dashboard_stats(
    auth: AuthUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let stats = StatsService::get_team_stats(&pool, auth.team_id).await?;
    Ok(HttpResponse::Ok().json(stats))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/dashboard/stats")
            .route(web::get().to(get_dashboard_stats))
    );
}
