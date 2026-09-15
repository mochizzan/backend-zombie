use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::leaderboard::{LeaderboardEntry, LeaderboardResponse, PlayerRank};

/// Upsert leaderboard entries for a user based on their current stats,
/// then recalculate ranks across all categories.
///
/// Called by user_stats::update_stats after a successful stats mutation.
/// This is `pub` so sibling services can call it:
/// `crate::services::leaderboard::upsert_for_user(pool, user_id)`
pub async fn upsert_for_user(pool: &DbPool, user_id: i64) -> Result<(), AppError> {
    // Fetch the user's current stats (if they don't exist, silently succeed)
    let stats = sqlx::query_as::<_, crate::models::user_stats::UserStats>(
        "SELECT * FROM user_stats WHERE user_id = ? AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let stats = match stats {
        Some(s) => s,
        None => return Ok(()),
    };

    // Upsert each stat category into the leaderboard
    let categories = [
        ("level", stats.level as i64),
        ("spent", stats.total_spent),
        ("playtime", stats.total_playtime),
        ("matches", stats.matches_played as i64),
    ];

    for (cat, score) in &categories {
        sqlx::query(
            r#"INSERT INTO leaderboard (user_id, category, score, `rank`) VALUES (?, ?, ?, 0)
            ON DUPLICATE KEY UPDATE score = VALUES(score)"#,
        )
        .bind(user_id)
        .bind(cat)
        .bind(score)
        .execute(pool)
        .await?;
    }

    // Recalculate dense ranks for every category
    recalculate_ranks(pool).await?;

    Ok(())
}

/// Recalculate DENSE_RANK for all four leaderboard categories.
///
/// Uses MariaDB window function OVER (ORDER BY score DESC) to assign ranks.
/// Ranks are dense: ties share a rank and the next rank is contiguous.
async fn recalculate_ranks(pool: &DbPool) -> Result<(), AppError> {
    let categories = ["level", "spent", "playtime", "matches"];

    for cat in &categories {
        sqlx::query(
            r#"UPDATE leaderboard l1
            INNER JOIN (
                SELECT user_id, DENSE_RANK() OVER (ORDER BY score DESC) AS new_rank
                FROM leaderboard
                WHERE category = ? AND deleted_at IS NULL
            ) l2 ON l1.user_id = l2.user_id AND l1.category = ?
            SET l1.`rank` = l2.new_rank"#,
        )
        .bind(cat)
        .bind(cat)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Fetch the leaderboard for a given category, optionally filtered by role.
///
/// Returns up to `limit` entries (max 100), ordered by rank ascending.
/// Joins against the users table to include player identity fields.
pub async fn get_leaderboard(
    pool: &DbPool,
    category: &str,
    limit: i64,
    role: Option<&str>,
) -> Result<LeaderboardResponse, AppError> {
    // Validate category to prevent injection
    let valid_categories = ["level", "spent", "playtime", "matches"];
    if !valid_categories.contains(&category) {
        return Err(AppError::Validation(format!(
            "Invalid category '{}'. Must be one of: level, spent, playtime, matches",
            category
        )));
    }

    let rows = if let Some(role_val) = role {
        sqlx::query_as::<_, (i32, i64, String, String, String, i64)>(
            r#"SELECT l.`rank`, l.user_id, u.player_id, u.username, u.nickname, l.score
            FROM leaderboard l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.category = ? AND l.deleted_at IS NULL AND u.deleted_at IS NULL
              AND u.role = ?
            ORDER BY l.`rank` ASC
            LIMIT ?"#,
        )
        .bind(category)
        .bind(role_val)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, (i32, i64, String, String, String, i64)>(
            r#"SELECT l.`rank`, l.user_id, u.player_id, u.username, u.nickname, l.score
            FROM leaderboard l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.category = ? AND l.deleted_at IS NULL AND u.deleted_at IS NULL
            ORDER BY l.`rank` ASC
            LIMIT ?"#,
        )
        .bind(category)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    let leaderboard: Vec<LeaderboardEntry> = rows
        .into_iter()
        .map(|(rank, user_id, player_id, username, nickname, score)| LeaderboardEntry {
            rank,
            user_id,
            player_id,
            username,
            nickname,
            score,
        })
        .collect();

    Ok(LeaderboardResponse {
        category: category.to_string(),
        leaderboard,
    })
}

/// Get a specific player's rank within a category.
///
/// Returns the player's rank, score, and the total number of ranked players
/// in that category. Returns NotFound if the player has no entry.
pub async fn get_player_rank(
    pool: &DbPool,
    category: &str,
    player_id: &str,
) -> Result<PlayerRank, AppError> {
    let valid_categories = ["level", "spent", "playtime", "matches"];
    if !valid_categories.contains(&category) {
        return Err(AppError::Validation(format!(
            "Invalid category '{}'. Must be one of: level, spent, playtime, matches",
            category
        )));
    }

    let row = sqlx::query_as::<_, (i32, i64)>(
        r#"SELECT l.`rank`, l.score
        FROM leaderboard l
        INNER JOIN users u ON u.id = l.user_id
        WHERE l.category = ? AND u.player_id = ? AND l.deleted_at IS NULL AND u.deleted_at IS NULL"#,
    )
    .bind(category)
    .bind(player_id)
    .fetch_optional(pool)
    .await?;

    let (rank, score) = row.ok_or_else(|| {
        AppError::NotFound(format!(
            "Player '{}' not found in '{}' leaderboard",
            player_id, category
        ))
    })?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM leaderboard WHERE category = ? AND deleted_at IS NULL",
    )
    .bind(category)
    .fetch_one(pool)
    .await?;

    Ok(PlayerRank {
        rank,
        score,
        total_players: total.0,
    })
}
