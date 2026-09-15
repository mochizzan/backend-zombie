# Roblox Game Backend — Design Spec

**Date:** 2026-09-16
**Status:** Approved
**Tech Stack:** Rust + Actix-web 4.15.0 + SQLx 0.8.2 + MariaDB 12

---

## 1. Overview

RESTful API backend for a Roblox game supporting two roles: **Survival** and **Killer**. The backend manages players, game statistics, shop, inventory, settings, and real-time leaderboards.

**Key Decisions:**
- **Framework:** Actix-web 4.15.0 (latest stable, built on Tokio)
- **Database:** MariaDB 12 via SQLx 0.8.2 (async, MySQL-compatible)
- **Auth:** API Key via `.env` (static, header-based)
- **Response Format:** Custom JSON Envelope (`{ success, data, error }`)
- **Pagination:** Cursor-based
- **Validation:** `validator` crate
- **Leaderboard:** Real-time materialized with `DENSE_RANK()`
- **Docker:** Multi-stage build for production

---

## 2. Database Schema

### Design Principles
- **Denormalized** — no heavy JOINs, duplicate data if needed for speed
- **BIGINT UNSIGNED** — future-proof IDs
- **Soft deletes** — `deleted_at` on all tables
- **ENUM** for fixed values — save storage, fast queries
- **Optimistic locking** — `version` column on user_stats

---

### 2.1 `users` — Core Player Table

```sql
CREATE TABLE users (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    player_id       VARCHAR(64) UNIQUE NOT NULL COMMENT 'Roblox player ID',
    username        VARCHAR(64) NOT NULL,
    nickname        VARCHAR(64) NOT NULL,
    role            ENUM('survival', 'killer') DEFAULT 'survival',
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    INDEX idx_player_id (player_id),
    INDEX idx_deleted_at (deleted_at)
);
```

---

### 2.2 `user_stats` — Game Statistics

```sql
CREATE TABLE user_stats (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    user_id         BIGINT UNSIGNED UNIQUE NOT NULL,
    exp             BIGINT UNSIGNED DEFAULT 0,
    fang            BIGINT UNSIGNED DEFAULT 0 COMMENT 'Premium currency',
    bcoin           BIGINT UNSIGNED DEFAULT 0 COMMENT 'Basic currency',
    level           INT UNSIGNED DEFAULT 1,
    total_playtime  BIGINT UNSIGNED DEFAULT 0 COMMENT 'Total seconds',
    matches_played  INT UNSIGNED DEFAULT 0,
    total_spent     BIGINT UNSIGNED DEFAULT 0 COMMENT 'Total currency spent',
    last_online_at  TIMESTAMP NULL,
    last_join_at    TIMESTAMP NULL,
    version         INT UNSIGNED DEFAULT 1 COMMENT 'Optimistic locking',
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_level (level DESC),
    INDEX idx_exp (exp DESC),
    INDEX idx_fang (fang DESC),
    INDEX idx_bcoin (bcoin DESC),
    INDEX idx_playtime (total_playtime DESC),
    INDEX idx_matches (matches_played DESC),
    INDEX idx_spent (total_spent DESC)
);
```

**Optimistic Locking:**
- `version` increments on every update
- Client sends `version` with update request
- If `version` doesn't match → conflict error (409)
- Prevents race condition from concurrent game server updates

---

### 2.3 `shop_items` — Role-Specific Shop

```sql
CREATE TABLE shop_items (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    role            ENUM('survival', 'killer') NOT NULL,
    category        ENUM('skill', 'item', 'emote') NOT NULL,
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    price           BIGINT UNSIGNED NOT NULL COMMENT 'Original price',
    discount_percent INT UNSIGNED DEFAULT 0 COMMENT '0-100',
    discounted_price BIGINT UNSIGNED NOT NULL COMMENT 'Stored for fast query',
    is_active       BOOLEAN DEFAULT TRUE,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    INDEX idx_role_category (role, category),
    INDEX idx_active (is_active),
    INDEX idx_discounted_price (discounted_price)
);
```

**Notes:**
- Single table for all roles (survival/killer) — simpler queries
- `discounted_price` stored (not computed) — fast price lookups
- `category` = skill, item, emote (fang & bcoin are currencies, not shop items)

---

### 2.4 `user_inventory` — User's Owned Items

```sql
CREATE TABLE user_inventory (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    user_id         BIGINT UNSIGNED NOT NULL,
    shop_item_id    BIGINT UNSIGNED NOT NULL,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (shop_item_id) REFERENCES shop_items(id) ON DELETE CASCADE,
    UNIQUE KEY uk_user_item (user_id, shop_item_id) COMMENT 'One row per user per item',
    INDEX idx_user_id (user_id),
    INDEX idx_shop_item_id (shop_item_id)
);
```

**Notes:**
- 1 shop item = 1 inventory row (no quantity)
- `uk_user_item` — unique constraint prevents duplicates

---

### 2.5 `user_settings` — Player Configuration

```sql
CREATE TABLE user_settings (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    user_id         BIGINT UNSIGNED NOT NULL,
    setting_key     VARCHAR(64) NOT NULL,
    setting_value   TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '0-255',
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY uk_user_setting (user_id, setting_key),
    INDEX idx_user_id (user_id)
);
```

**Example Settings:**

| Key | Range | Description |
|-----|-------|-------------|
| `music_volume` | 0-100 | Music volume |
| `sfx_volume` | 0-100 | Sound effects volume |
| `sensitivity` | 1-100 | Mouse/camera sensitivity |
| `graphics_quality` | 0-2 | 0=Low, 1=Medium, 2=High |
| `language` | 0-5 | 0=EN, 1=ID, 2=JP, etc. |
| `show_damage` | 0-1 | 0=Off, 1=On |
| `auto_attack` | 0-1 | 0=Off, 1=On |

---

### 2.6 `leaderboard` — Real-Time Materialized

```sql
CREATE TABLE leaderboard (
    id              BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    user_id         BIGINT UNSIGNED NOT NULL,
    category        ENUM('level', 'spent', 'playtime', 'matches') NOT NULL,
    score           BIGINT UNSIGNED NOT NULL,
    rank            INT UNSIGNED NOT NULL,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY uk_user_category (user_id, category),
    INDEX idx_category_score (category, score DESC),
    INDEX idx_category_rank (category, rank)
);
```

**Real-Time Update Strategy:**
1. When `user_stats` changes → upsert into `leaderboard`
2. After upsert → recalculate ranks using `DENSE_RANK()`
3. No gaps in ranking (1, 1, 2, 3, 3, 4...)

**Rank Calculation (DENSE_RANK — no gaps):**
```sql
UPDATE leaderboard l1
INNER JOIN (
    SELECT user_id,
           DENSE_RANK() OVER (ORDER BY score DESC) as new_rank
    FROM leaderboard
    WHERE category = ? AND deleted_at IS NULL
) l2 ON l1.user_id = l2.user_id AND l1.category = ?
SET l1.rank = l2.new_rank;
```

**Rank Logic:**
- Score 100, 100, 95, 90, 85 → Rank: 1, 1, 2, 3, 4 (no gaps)
- Uses `DENSE_RANK()` not `RANK()` — rank numbers always consecutive

---

### 2.7 Entity Relationship

```
┌─────────────┐      1:1      ┌──────────────┐
│    users     │──────────────│  user_stats   │
│             │              │              │
│  player_id  │              │  exp, level  │
│  username   │              │  fang, bcoin │
│  nickname   │              │  playtime    │
│  role       │              │  matches     │
└──────┬──────┘              └──────────────┘
       │
       │ 1:N
       ▼
┌─────────────────┐   N:1    ┌──────────────┐
│ user_inventory   │─────────│  shop_items   │
│ (no quantity)    │         │              │
└─────────────────┘         │  role        │
       │                    │  category    │
       │ 1:N                │  price       │
       ▼                    └──────────────┘
┌─────────────────┐
│ user_settings   │
│ (tinyint 0-255) │
└─────────────────┘

┌─────────────┐
│ leaderboard  │ ← materialized, updated real-time
│ (rank-based) │
└─────────────┘
```

---

## 3. API Endpoints

**Base URL:** `http://localhost:3000/api/v1`
**Auth:** Header `X-API-Key: <api-key>` (from `.env`)
**Content-Type:** `application/json`

---

### 3.1 Users

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| `POST` | `/users` | Register/login player (upsert) | `{ player_id, username, nickname, role? }` |
| `GET` | `/users/:player_id` | Get player profile | — |
| `PATCH` | `/users/:player_id` | Update profile | `{ username?, nickname?, role? }` |
| `DELETE` | `/users/:player_id` | Soft delete player | — |

---

### 3.2 User Stats

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| `GET` | `/users/:player_id/stats` | Get all stats | — |
| `PATCH` | `/users/:player_id/stats` | Update stats (game server) | `{ exp?, fang?, bcoin?, level?, total_playtime?, matches_played?, version }` |
| `POST` | `/users/:player_id/stats/online` | Update last_online_at | — |
| `POST` | `/users/:player_id/stats/join` | Update last_join_at | — |

**Notes:**
- `PATCH /stats` = game server calls this on every stat change
- `total_spent` auto-calculated from inventory purchases
- `version` required for optimistic locking

---

### 3.3 Shop

| Method | Endpoint | Description | Query Params |
|--------|----------|-------------|--------------|
| `GET` | `/shop` | List shop items | `role`, `category`, `cursor`, `limit` |
| `GET` | `/shop/:id` | Get shop item detail | — |
| `POST` | `/shop` | Add shop item (admin) | `{ role, category, name, description, price, discount_percent }` |
| `PATCH` | `/shop/:id` | Update shop item (admin) | `{ name?, price?, discount_percent?, is_active? }` |
| `DELETE` | `/shop/:id` | Soft delete shop item (admin) | — |

**Query Params:**
- `role` = `survival` | `killer`
- `category` = `skill` | `item` | `emote`
- `cursor` = cursor ID for pagination
- `limit` = items per page (default 20, max 100)

---

### 3.4 Inventory

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| `GET` | `/users/:player_id/inventory` | List inventory | Query: `category`, `cursor`, `limit` |
| `POST` | `/users/:player_id/inventory` | Purchase item | `{ shop_item_id }` |
| `DELETE` | `/users/:player_id/inventory/:item_id` | Remove item | — |
| `GET` | `/users/:player_id/inventory/has/:shop_item_id` | Check ownership | — |

**Notes:**
- 1 shop item = 1 inventory row (no quantity)
- Purchase = INSERT into inventory (not quantity update)

---

### 3.5 Settings

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| `GET` | `/users/:player_id/settings` | Get all settings | — |
| `PUT` | `/users/:player_id/settings` | Update multiple settings | `{ settings: { key: value } }` |
| `PATCH` | `/users/:player_id/settings/:key` | Update single setting | `{ value: 50 }` |
| `DELETE` | `/users/:player_id/settings/:key` | Delete setting | — |

**Notes:**
- All values are `TINYINT UNSIGNED` (0-255)
- Key-value pattern for flexibility

---

### 3.6 Leaderboard

| Method | Endpoint | Description | Query Params |
|--------|----------|-------------|--------------|
| `GET` | `/leaderboard/:category` | Get top players | `limit`, `role` |
| `GET` | `/leaderboard/:category/me` | Get player rank | `player_id` |

**Categories:** `level`, `spent`, `playtime`, `matches`

**Query Params:**
- `limit` = top players count (default 10, max 100)
- `role` = filter by role (`survival` | `killer`), optional

---

### 3.7 Summary

```
POST   /api/v1/users                          — Register/login
GET    /api/v1/users/:player_id               — Get profile
PATCH  /api/v1/users/:player_id               — Update profile
DELETE /api/v1/users/:player_id               — Delete player

GET    /api/v1/users/:player_id/stats         — Get stats
PATCH  /api/v1/users/:player_id/stats         — Update stats
POST   /api/v1/users/:player_id/stats/online  — Update online
POST   /api/v1/users/:player_id/stats/join    — Update join

GET    /api/v1/shop                           — List shop items
GET    /api/v1/shop/:id                       — Get shop item
POST   /api/v1/shop                           — Add shop item (admin)
PATCH  /api/v1/shop/:id                       — Update shop item (admin)
DELETE /api/v1/shop/:id                       — Delete shop item (admin)

GET    /api/v1/users/:player_id/inventory     — List inventory
POST   /api/v1/users/:player_id/inventory     — Purchase item
DELETE /api/v1/users/:player_id/inventory/:item_id — Remove item
GET    /api/v1/users/:player_id/inventory/has/:shop_item_id — Check ownership

GET    /api/v1/users/:player_id/settings      — Get settings
PUT    /api/v1/users/:player_id/settings      — Update multiple settings
PATCH  /api/v1/users/:player_id/settings/:key — Update single setting
DELETE /api/v1/users/:player_id/settings/:key — Delete setting

GET    /api/v1/leaderboard/:category          — Get top players
GET    /api/v1/leaderboard/:category/me       — Get player rank
```

**Total: 22 endpoints**

---

## 4. Response Format

### Success (Single)
```json
{
  "success": true,
  "data": { ... }
}
```

### Success (List with Cursor Pagination)
```json
{
  "success": true,
  "data": [ ... ],
  "meta": {
    "cursor": "eyJpZCI6MX0=",
    "has_more": true
  }
}
```

### Error
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Username already exists"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid input |
| `UNAUTHORIZED` | 401 | Wrong/missing API key |
| `NOT_FOUND` | 404 | Data not found |
| `CONFLICT` | 409 | Duplicate data |
| `VERSION_CONFLICT` | 409 | Optimistic lock failed |
| `INTERNAL_ERROR` | 500 | Server error |

---

## 5. Project Structure

```
backend/
├── Cargo.toml
├── .env                          # API_KEY, DATABASE_URL
├── .env.example                  # Template .env
├── Dockerfile                    # Multi-stage build
├── docker-compose.yml            # Local dev (MariaDB 12 + API)
├── docker-compose.prod.yml       # Production
│
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_user_stats.sql
│   ├── 003_create_shop_items.sql
│   ├── 004_create_user_inventory.sql
│   ├── 005_create_user_settings.sql
│   └── 006_create_leaderboard.sql
│
├── src/
│   ├── main.rs                   # Entry point, app setup
│   ├── config.rs                 # Environment config (.env)
│   ├── errors.rs                 # AppError type
│   ├── db.rs                     # Database pool setup
│   │
│   ├── models/
│   │   ├── mod.rs                # Re-export
│   │   ├── user.rs               # User, CreateUser, UpdateUser
│   │   ├── user_stats.rs         # UserStats, UpdateStats
│   │   ├── shop_item.rs          # ShopItem, CreateShopItem
│   │   ├── user_inventory.rs     # UserInventory
│   │   ├── user_settings.rs      # UserSettings
│   │   └── leaderboard.rs        # Leaderboard
│   │
│   ├── handlers/
│   │   ├── mod.rs                # Re-export
│   │   ├── user.rs               # User endpoints
│   │   ├── user_stats.rs         # Stats endpoints
│   │   ├── shop.rs               # Shop endpoints
│   │   ├── inventory.rs          # Inventory endpoints
│   │   ├── settings.rs           # Settings endpoints
│   │   └── leaderboard.rs        # Leaderboard endpoints
│   │
│   ├── services/
│   │   ├── mod.rs                # Re-export
│   │   ├── user.rs               # User business logic
│   │   ├── user_stats.rs         # Stats business logic
│   │   ├── shop.rs               # Shop business logic
│   │   ├── inventory.rs          # Inventory business logic
│   │   ├── settings.rs           # Settings business logic
│   │   └── leaderboard.rs        # Leaderboard business logic
│   │
│   ├── routes.rs                 # Route registration
│   └── middleware/
│       └── apikey.rs             # API key validation
│
├── tests/
│   ├── integration/
│   │   ├── user_test.rs
│   │   ├── stats_test.rs
│   │   ├── shop_test.rs
│   │   ├── inventory_test.rs
│   │   ├── settings_test.rs
│   │   └── leaderboard_test.rs
│   └── helpers.rs                # Test utilities
│
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-09-16-roblox-backend-design.md
```

---

## 6. Request Flow

```
Client (Roblox Game)
        │
        ▼
┌─────────────────┐
│  API Key Check   │ ← middleware/apikey.rs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Handler Layer   │ ← handlers/*.rs
│  (HTTP logic)    │   - Parse request
│                  │   - Validate input
└────────┬────────┘   - Return response
         │
         ▼
┌─────────────────┐
│  Service Layer   │ ← services/*.rs
│  (Business)      │   - Business rules
│                  │   - Orchestration
└────────┬────────┘   - Call repositories
         │
         ▼
┌─────────────────┐
│  Database Layer  │ ← sqlx queries (inline)
│  (SQL queries)   │   - SELECT, INSERT, UPDATE, DELETE
└─────────────────┘   - MariaDB 12
```

---

## 7. Docker Setup

### Local Development

```bash
# Run MariaDB + API
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop
docker-compose down
```

### docker-compose.yml (Local)

```yaml
services:
  db:
    image: mariadb:12
    environment:
      MYSQL_ROOT_PASSWORD: rootpassword
      MYSQL_DATABASE: roblox_game
      MYSQL_USER: gameuser
      MYSQL_PASSWORD: gamepassword
    ports:
      - "3306:3306"
    volumes:
      - mariadb_data:/var/lib/mysql

  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: mysql://gameuser:gamepassword@db:3306/roblox_game
      API_KEY: your-secret-api-key
      RUST_LOG: debug
    depends_on:
      - db

volumes:
  mariadb_data:
```

### docker-compose.prod.yml (Production)

```yaml
services:
  db:
    image: mariadb:12
    environment:
      MYSQL_ROOT_PASSWORD: ${DB_ROOT_PASSWORD}
      MYSQL_DATABASE: roblox_game
      MYSQL_USER: ${DB_USER}
      MYSQL_PASSWORD: ${DB_PASSWORD}
    volumes:
      - mariadb_prod_data:/var/lib/mysql
    restart: always

  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: mysql://${DB_USER}:${DB_PASSWORD}@db:3306/roblox_game
      API_KEY: ${API_KEY}
      RUST_LOG: warn
    depends_on:
      - db
    restart: always

volumes:
  mariadb_prod_data:
```

---

## 8. Configuration

### .env

```env
# Database
DATABASE_URL=mysql://gameuser:gamepassword@localhost:3306/roblox_game

# API
API_KEY=your-secret-api-key-here
API_PORT=3000

# Logging
RUST_LOG=info
```

### .env.example

```env
DATABASE_URL=mysql://gameuser:gamepassword@localhost:3306/roblox_game
API_KEY=change-me-to-a-secure-key
API_PORT=3000
RUST_LOG=info
```

---

## 9. Dependencies

### Cargo.toml

```toml
[package]
name = "roblox-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
actix-web = "4.15.0"
actix-rt = "2.5.0"

# Database
sqlx = { version = "0.8.2", features = ["runtime-tokio", "mysql", "chrono"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Environment
dotenvy = "0.15"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-actix-web = "0.7"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Base64 (for cursor pagination)
base64 = "0.22"

[dev-dependencies]
sqlx = { version = "0.8.2", features = ["runtime-tokio", "mysql", "chrono", "offline"] }
```

---

## 10. Testing Strategy

| Test Type | Scope | Tool |
|-----------|-------|------|
| Unit Test | Business logic (services) | `cargo test` |
| Integration Test | API endpoints + MariaDB | `cargo test --test integration` |

### Integration Test Flow
1. Spin up MariaDB Docker container
2. Run migrations
3. Send HTTP requests to API
4. Assert responses
5. Cleanup

---

## 11. Performance Optimizations

| Technique | Application | Benefit |
|-----------|-------------|---------|
| Denormalized | `total_spent` in user_stats | No JOIN for spending calculation |
| Stored computed | `discounted_price` in shop_items | Fast price lookups |
| Materialized | `leaderboard` table | < 1ms leaderboard queries |
| Optimistic locking | `version` column | Prevent race conditions |
| Composite index | `idx_role_category` | Fast shop queries by role+category |
| Soft deletes | `deleted_at` on all tables | Data history preserved |

---

## 12. Future Considerations

- **Redis caching** — For hot data (leaderboard, user stats)
- **Rate limiting** — API key-based rate limiting
- **Audit trail** — Separate audit log table
- **Webhook** — Event notifications to game server
- **Batch operations** — Bulk stat updates
- **Read replicas** — For read-heavy endpoints
