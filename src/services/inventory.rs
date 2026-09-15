use sqlx::MySqlPool;
use crate::errors::AppError;
use crate::models::user_inventory::{UserInventory, PurchaseItem};
use crate::models::shop_item::ShopItem;
use base64::{Engine, engine::general_purpose};

/// List inventory items for a player with optional category filter and cursor pagination.
/// Returns (items_with_details, next_cursor, has_more).
pub async fn list_inventory(
    pool: &MySqlPool,
    player_id: &str,
    category: Option<&str>,
    cursor: Option<&str>,
    limit: i64,
) -> Result<(Vec<(UserInventory, ShopItem)>, Option<String>, bool), AppError> {
    let last_id = cursor
        .and_then(|c| general_purpose::STANDARD.decode(c).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse::<i64>().ok());

    let fetch_limit = limit + 1;

    let mut rows = match (category, last_id) {
        (Some(cat), Some(lid)) => {
            sqlx::query_as::<_, UserInventory>(
                r#"SELECT ui.* FROM user_inventory ui
                INNER JOIN users u ON u.id = ui.user_id
                INNER JOIN shop_items si ON si.id = ui.shop_item_id
                WHERE u.player_id = ? AND u.deleted_at IS NULL AND ui.deleted_at IS NULL
                    AND si.deleted_at IS NULL AND si.category = ? AND ui.id > ?
                ORDER BY ui.id ASC LIMIT ?"#,
            )
            .bind(player_id)
            .bind(cat)
            .bind(lid)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await?
        }
        (Some(cat), None) => {
            sqlx::query_as::<_, UserInventory>(
                r#"SELECT ui.* FROM user_inventory ui
                INNER JOIN users u ON u.id = ui.user_id
                INNER JOIN shop_items si ON si.id = ui.shop_item_id
                WHERE u.player_id = ? AND u.deleted_at IS NULL AND ui.deleted_at IS NULL
                    AND si.deleted_at IS NULL AND si.category = ?
                ORDER BY ui.id ASC LIMIT ?"#,
            )
            .bind(player_id)
            .bind(cat)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await?
        }
        (None, Some(lid)) => {
            sqlx::query_as::<_, UserInventory>(
                r#"SELECT ui.* FROM user_inventory ui
                INNER JOIN users u ON u.id = ui.user_id
                WHERE u.player_id = ? AND u.deleted_at IS NULL AND ui.deleted_at IS NULL
                    AND ui.id > ?
                ORDER BY ui.id ASC LIMIT ?"#,
            )
            .bind(player_id)
            .bind(lid)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, UserInventory>(
                r#"SELECT ui.* FROM user_inventory ui
                INNER JOIN users u ON u.id = ui.user_id
                WHERE u.player_id = ? AND u.deleted_at IS NULL AND ui.deleted_at IS NULL
                ORDER BY ui.id ASC LIMIT ?"#,
            )
            .bind(player_id)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await?
        }
    };

    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.pop();
    }

    let next_cursor = rows
        .last()
        .map(|r| general_purpose::STANDARD.encode(r.id.to_string()));

    // Fetch shop item details for each inventory row
    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        // Skip silently if shop item has been soft-deleted
        if let Ok(item) = crate::services::shop::get_item(pool, row.shop_item_id).await {
            results.push((row, item));
        }
    }

    Ok((results, next_cursor, has_more))
}

/// Purchase a shop item for a player.
/// Validates user exists, shop item is active, inserts into inventory,
/// and updates total_spent in user_stats.
pub async fn purchase_item(
    pool: &MySqlPool,
    player_id: &str,
    data: PurchaseItem,
) -> Result<UserInventory, AppError> {
    // Verify user exists
    let user = crate::services::user::get_user(pool, player_id).await?;

    // Verify shop item exists and is active
    let item = crate::services::shop::get_item(pool, data.shop_item_id).await?;
    if !item.is_active {
        return Err(AppError::Validation(
            "Shop item is not active".to_string(),
        ));
    }

    // Insert inventory (unique key on user_id + shop_item_id prevents duplicates)
    sqlx::query("INSERT INTO user_inventory (user_id, shop_item_id) VALUES (?, ?)")
        .bind(user.id)
        .bind(data.shop_item_id)
        .execute(pool)
        .await?;

    // Update total_spent in user_stats
    sqlx::query(
        "UPDATE user_stats SET total_spent = total_spent + ? WHERE user_id = ? AND deleted_at IS NULL",
    )
    .bind(item.discounted_price)
    .bind(user.id)
    .execute(pool)
    .await?;

    // Fetch the created inventory entry
    let inv = sqlx::query_as::<_, UserInventory>(
        "SELECT * FROM user_inventory WHERE user_id = ? AND shop_item_id = ? AND deleted_at IS NULL",
    )
    .bind(user.id)
    .bind(data.shop_item_id)
    .fetch_one(pool)
    .await?;

    Ok(inv)
}

/// Soft-delete an inventory item. Only the owning user can remove their own items.
pub async fn remove_item(
    pool: &MySqlPool,
    player_id: &str,
    inventory_id: i64,
) -> Result<(), AppError> {
    // Verify user exists (and isn't soft-deleted)
    let user = crate::services::user::get_user(pool, player_id).await?;

    let result = sqlx::query(
        "UPDATE user_inventory SET deleted_at = NOW() WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(inventory_id)
    .bind(user.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Inventory item not found".to_string(),
        ));
    }
    Ok(())
}

/// Check if a player owns a specific shop item.
pub async fn has_item(
    pool: &MySqlPool,
    player_id: &str,
    shop_item_id: i64,
) -> Result<bool, AppError> {
    let user = crate::services::user::get_user(pool, player_id).await?;

    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_inventory ui
        INNER JOIN users u ON u.id = ui.user_id
        WHERE u.player_id = ? AND ui.shop_item_id = ? AND ui.deleted_at IS NULL AND u.deleted_at IS NULL",
    )
    .bind(player_id)
    .bind(shop_item_id)
    .fetch_one(pool)
    .await?;

    Ok(row > 0)
}
