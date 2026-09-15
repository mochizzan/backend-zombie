use sqlx::MySqlPool;
use crate::errors::AppError;
use crate::models::user::{User, CreateUser, UpdateUser};

pub async fn create_user(pool: &MySqlPool, data: CreateUser) -> Result<User, AppError> {
    let role = data.role.unwrap_or_else(|| "survival".to_string());
    // Validate role
    if role != "survival" && role != "killer" {
        return Err(AppError::Validation(
            "Role must be 'survival' or 'killer'".to_string(),
        ));
    }
    // Upsert by player_id
    sqlx::query(
        r#"INSERT INTO users (player_id, username, nickname, role)
        VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            username = VALUES(username),
            nickname = VALUES(nickname),
            role = VALUES(role),
            deleted_at = NULL"#,
    )
    .bind(&data.player_id)
    .bind(&data.username)
    .bind(&data.nickname)
    .bind(&role)
    .execute(pool)
    .await?;
    // Fetch the upserted user
    let user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE player_id = ? AND deleted_at IS NULL")
            .bind(&data.player_id)
            .fetch_one(pool)
            .await?;
    Ok(user)
}

pub async fn get_user(pool: &MySqlPool, player_id: &str) -> Result<User, AppError> {
    let user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE player_id = ? AND deleted_at IS NULL")
            .bind(player_id)
            .fetch_optional(pool)
            .await?;
    user.ok_or_else(|| AppError::NotFound(format!("User '{}' not found", player_id)))
}

pub async fn update_user(
    pool: &MySqlPool,
    player_id: &str,
    data: UpdateUser,
) -> Result<User, AppError> {
    let existing = get_user(pool, player_id).await?;
    let username = data.username.unwrap_or(existing.username);
    let nickname = data.nickname.unwrap_or(existing.nickname);
    let role = data.role.unwrap_or(existing.role);
    if role != "survival" && role != "killer" {
        return Err(AppError::Validation(
            "Role must be 'survival' or 'killer'".to_string(),
        ));
    }
    sqlx::query(
        "UPDATE users SET username = ?, nickname = ?, role = ? WHERE player_id = ? AND deleted_at IS NULL",
    )
    .bind(&username)
    .bind(&nickname)
    .bind(&role)
    .bind(player_id)
    .execute(pool)
    .await?;
    get_user(pool, player_id).await
}

pub async fn delete_user(pool: &MySqlPool, player_id: &str) -> Result<(), AppError> {
    let result =
        sqlx::query("UPDATE users SET deleted_at = NOW() WHERE player_id = ? AND deleted_at IS NULL")
            .bind(player_id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "User '{}' not found",
            player_id
        )));
    }
    // Also soft-delete stats, inventory, settings, leaderboard
    sqlx::query(
        "UPDATE user_stats SET deleted_at = NOW() WHERE user_id = (SELECT id FROM users WHERE player_id = ?)",
    )
    .bind(player_id)
    .execute(pool)
    .await
    .ok();
    Ok(())
}
