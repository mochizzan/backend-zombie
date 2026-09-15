CREATE TABLE IF NOT EXISTS shop_items (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    role ENUM('survival', 'killer', 'both') NOT NULL,
    category VARCHAR(64) NOT NULL,
    name VARCHAR(128) NOT NULL,
    description TEXT NULL,
    price BIGINT NOT NULL,
    discount_percent INT NULL DEFAULT 0,
    discounted_price BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    INDEX idx_shop_items_role_category (role, category),
    INDEX idx_shop_items_deleted_at (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
