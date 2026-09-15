use actix_web::{web, HttpResponse};

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::success_response;
use crate::services;

#[derive(serde::Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
    pub role: Option<String>,
}

/// GET /api/v1/leaderboard/{category}
///
/// Fetches the top players in a given stat category.
/// Query params: ?limit=20&role=survival
pub async fn get_leaderboard(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    query: web::Query<LeaderboardQuery>,
) -> Result<HttpResponse, AppError> {
    let category = path.into_inner();
    let limit = query.limit.unwrap_or(20).min(100);

    let resp =
        services::leaderboard::get_leaderboard(pool.get_ref(), &category, limit, query.role.as_deref())
            .await?;

    Ok(success_response(resp))
}

/// GET /api/v1/leaderboard/{category}/{player_id}
///
/// Fetches a specific player's rank within a stat category.
pub async fn get_player_rank(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (category, player_id) = path.into_inner();

    let rank =
        services::leaderboard::get_player_rank(pool.get_ref(), &category, &player_id).await?;

    Ok(success_response(rank))
}
