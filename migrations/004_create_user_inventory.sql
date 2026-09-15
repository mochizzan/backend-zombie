CREATE TABLE IF NOT EXISTS user_inventory (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    shop_item_id BIGINT NOT NULL,
    equipped BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    UNIQUE KEY unique_user_item (user_id, shop_item_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (shop_item_id) REFERENCES shop_items(id) ON DELETE CASCADE,
    INDEX idx_inventory_user_id (user_id),
    INDEX idx_inventory_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
