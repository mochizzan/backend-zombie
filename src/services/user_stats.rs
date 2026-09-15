use crate::errors::AppError;
use crate::models::user_stats::{UpdateStats, UserStats};
use sqlx::MySqlPool;

pub async fn get_stats(pool: &MySqlPool, player_id: &str) -> Result<UserStats, AppError> {
    let stats = sqlx::query_as::<_, UserStats>(
        r#"SELECT us.* FROM user_stats us
        INNER JOIN users u ON u.id = us.user_id
        WHERE u.player_id = ? AND u.deleted_at IS NULL AND us.deleted_at IS NULL"#,
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;
    stats.ok_or_else(|| AppError::NotFound(format!("Stats for '{}' not found", player_id)))
}

pub async fn update_stats(
    pool: &MySqlPool,
    player_id: &str,
    data: UpdateStats,
) -> Result<UserStats, AppError> {
    let current = get_stats(pool, player_id).await?;

    if let Some(version) = data.version {
        if version != current.version {
            return Err(AppError::VersionConflict(format!(
                "Expected version {}, got {}",
                current.version, version
            )));
        }
    }

    let mut sets = Vec::new();
    if data.exp.is_some() {
        sets.push("exp = ?");
    }
    if data.fang.is_some() {
        sets.push("fang = ?");
    }
    if data.bcoin.is_some() {
        sets.push("bcoin = ?");
    }
    if data.level.is_some() {
        sets.push("level = ?");
    }
    if data.total_playtime.is_some() {
        sets.push("total_playtime = ?");
    }
    if data.matches_played.is_some() {
        sets.push("matches_played = ?");
    }
    sets.push("version = version + 1");

    if sets.is_empty() {
        return Ok(current);
    }

    let sql = format!(
        "UPDATE user_stats SET {} WHERE user_id = ? AND deleted_at IS NULL",
        sets.join(", ")
    );

    let mut query = sqlx::query(&sql);
    if let Some(v) = data.exp {
        query = query.bind(v);
    }
    if let Some(v) = data.fang {
        query = query.bind(v);
    }
    if let Some(v) = data.bcoin {
        query = query.bind(v);
    }
    if let Some(v) = data.level {
        query = query.bind(v);
    }
    if let Some(v) = data.total_playtime {
        query = query.bind(v);
    }
    if let Some(v) = data.matches_played {
        query = query.bind(v);
    }
    query = query.bind(current.user_id);
    query.execute(pool).await?;

    crate::services::leaderboard::upsert_for_user(pool, current.user_id)
        .await
        .ok();

    get_stats(pool, player_id).await
}

pub async fn update_online(pool: &MySqlPool, player_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE user_stats us INNER JOIN users u ON u.id = us.user_id \
         SET us.last_online_at = NOW() \
         WHERE u.player_id = ? AND u.deleted_at IS NULL",
    )
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_join(pool: &MySqlPool, player_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE user_stats us INNER JOIN users u ON u.id = us.user_id \
         SET us.last_join_at = NOW() \
         WHERE u.player_id = ? AND u.deleted_at IS NULL",
    )
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}
