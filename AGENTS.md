# Repository Guidelines

## Project Overview

Rust REST API backend for a Roblox game. Manages players, game stats, shop, inventory, settings, and real-time leaderboards. Built with Actix-web 4.15.0 + SQLx 0.8.2 + MariaDB 11.

**Repository:** `mochizzan/backend-zombie` on GitHub

---

## Architecture & Data Flow

**Pattern:** Service Layer (Handler → Service → SQLx → MariaDB)

```
HTTP Request
  → ApiKey Middleware (validates X-API-Key header)
    → Handler (extracts params, validates input, calls service)
      → Service (business logic, DB queries via SQLx)
        → MariaDB (via sqlx::MySqlPool)
      ← Result<T, AppError>
    ← HttpResponse (JSON envelope)
  ← HTTP Response
```

**Key Design Decisions:**
- **No trait abstractions** — free async functions over `&MySqlPool`
- **Dynamic SQL** — `format!()` for variable WHERE clauses (role/category filters)
- **Cursor pagination** — base64-encoded IDs with fetch-limit+1 pattern
- **Soft deletes** — `deleted_at` column on all tables
- **Optimistic locking** — `version` field on `user_stats`
- **Upsert pattern** — `INSERT ON DUPLICATE KEY UPDATE` for users/settings
- **Fetch-after-write** — query DB after insert/update to return fresh data

---

## Key Directories

```
src/
├── main.rs              # Entry point: config, pool, migrations, server
├── config.rs            # Config struct (DATABASE_URL, API_KEY, API_PORT)
├── db.rs                # DbPool type alias, init_pool()
├── errors.rs            # AppError enum (7 variants) with ResponseError
├── routes.rs            # All 22 route registrations
├── models/              # DB structs (FromRow) + request/response structs
├── handlers/            # HTTP handlers (extraction, validation, response)
├── services/            # Business logic (free async functions)
├── middleware/           # API key auth middleware (Transform+Service)
├── migrations/          # 6 SQL migration files (CREATE TABLE)
└── tests/integration/   # Integration test skeleton (#[ignore]d)
```

---

## Development Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Run
cargo run                      # Start server (requires .env with DATABASE_URL)
cargo run --release            # Production build + run

# Test
cargo test                     # Compile tests (all #[ignore]d, need DB)
cargo test -- --ignored        # Run ignored tests (requires MariaDB)
cargo test user_test           # Run specific test file

# Database (via Docker)
docker-compose up -d db        # Start MariaDB
docker-compose up -d           # Start DB + API

# Lint/Check
cargo check                    # Fast compile check
cargo clippy                   # Lint (if installed)
cargo fmt                      # Format code
```

---

## Code Conventions & Common Patterns

### Error Handling
```rust
// AppError enum variants map to HTTP status codes:
// NotFound → 404, Validation → 400, Conflict/VersionConflict → 409
// Unauthorized → 401, Internal/Database → 500

// Services return Result<T, AppError>
pub async fn get_user(pool: &MySqlPool, player_id: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(...)
        .fetch_optional(pool)
        .await?;  // sqlx::Error auto-converts to AppError via From impl
    user.ok_or_else(|| AppError::NotFound(format!("User '{}' not found", player_id)))
}
```

### JSON Response Envelope
```rust
// All responses use consistent envelope:
// Success: { "success": true, "data": <T> }
// List:    { "success": true, "data": [...], "meta": { "cursor": "...", "has_more": bool } }
// Error:   { "success": false, "error": { "code": "...", "message": "..." } }

use crate::models::{success_response, created_response, list_response, no_content_response};

pub async fn handler(...) -> Result<HttpResponse, AppError> {
    Ok(success_response(data))           // 200
    Ok(created_response(data))           // 201
    Ok(list_response(items, cursor, has_more))  // 200 with meta
    Ok(no_content_response())            // 204
}
```

### Handler Pattern
```rust
pub async fn get_profile(
    pool: web::Data<DbPool>,           // Injected state
    path: web::Path<String>,           // Path params
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let user = services::user::get_user(pool.get_ref(), &player_id).await?;
    Ok(success_response(user))
}
```

### Service Pattern
```rust
// Free async functions over &MySqlPool (no traits, no structs)
pub async fn create_user(pool: &MySqlPool, data: CreateUser) -> Result<User, AppError> {
    sqlx::query("INSERT INTO users ...")
        .bind(&data.player_id)
        .execute(pool)
        .await?;
    // Fetch-after-write pattern
    get_user(pool, &data.player_id).await
}
```

### Cursor Pagination
```rust
use base64::{Engine, engine::general_purpose};

// Decode cursor to get last_id
let last_id = cursor
    .and_then(|c| general_purpose::STANDARD.decode(c).ok())
    .and_then(|b| String::from_utf8(b).ok())
    .and_then(|s| s.parse::<i64>().ok());

// Fetch limit + 1 to detect has_more
let fetch_limit = limit + 1;
let mut items = sqlx::query_as::<_, T>("SELECT * WHERE id > ? ORDER BY id ASC LIMIT ?")
    .bind(last_id).bind(fetch_limit)
    .fetch_all(pool).await?;

let has_more = items.len() as i64 > limit;
if has_more { items.pop(); }
let next_cursor = items.last().map(|i| general_purpose::STANDARD.encode(i.id.to_string()));
```

### Validation
```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, max = 64))]
    pub player_id: String,
    // ...
}

// In handler:
body.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

### Module Exports
```rust
// src/models/mod.rs re-exports all sub-modules + response helpers
pub mod user;
pub mod user_stats;
pub mod shop_item;
// ... plus success_response(), created_response(), list_response(), no_content_response()
```

---

## Important Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point: config, pool, migrations, server setup |
| `src/routes.rs` | All 22 route registrations (single `configure()` fn) |
| `src/errors.rs` | AppError enum + ResponseError impl (JSON error envelope) |
| `src/models/mod.rs` | Response helpers + module re-exports |
| `src/middleware/apikey.rs` | API key auth middleware (X-API-Key header) |
| `Cargo.toml` | Dependencies, edition, features |
| `Dockerfile` | Multi-stage build (rust:1.85 → debian:bookworm-slim) |
| `docker-compose.yml` | Local dev: MariaDB 11 + API |
| `.env` | Required env vars (DATABASE_URL, API_KEY, API_PORT, RUST_LOG) |
| `migrations/` | 6 SQL migration files (run on startup via sqlx::migrate!) |

---

## Runtime/Tooling Preferences

- **Language:** Rust (edition 2024)
- **Runtime:** Actix-web 4.15.0 with Tokio (via actix-rt 2.5.0)
- **Database:** MariaDB 11 (via SQLx 0.8.2, MySQL compatible)
- **Package manager:** Cargo (Cargo.lock committed)
- **Docker:** Multi-stage build, non-root user in runtime
- **Environment:** dotenvy for .env loading
- **No workspace:** Single-crate project

---

## Testing & QA

**Framework:** `actix_web::test` with `actix_rt::test` macro

**Status:** Skeletal — all 18 tests are `#[ignore]`d (require running MariaDB)

**Known Issues:**
- Missing `tests/integration.rs` glue file (Cargo won't compile subdirectory tests)
- `tests/helpers.rs` unused (API_KEY duplicated in each test file)
- Stub app setup in `user_test.rs` doesn't mount real routes

**Running Tests:**
```bash
# Compile only (fast, no DB needed)
cargo test

# Run with DB (requires MariaDB running)
cargo test -- --ignored

# Run specific test
cargo test test_register_and_get_user -- --ignored
```

**Test File Structure:**
```
tests/
├── helpers.rs                    # Shared constants (API_KEY)
└── integration/
    ├── user_test.rs              # 3 tests (skeleton with real logic)
    ├── shop_test.rs              # 3 tests (placeholders)
    ├── stats_test.rs             # 3 tests (placeholders)
    ├── inventory_test.rs         # 3 tests (placeholders)
    ├── settings_test.rs          # 3 tests (placeholders)
    └── leaderboard_test.rs       # 3 tests (placeholders)
```

**Test Naming:** `test_<action>_<resource>` (e.g., `test_register_and_get_user`)

---

## API Endpoints (22 total)

| Group | Method | Path | Handler |
|-------|--------|------|---------|
| Users | POST | `/api/v1/users` | `register` |
| Users | GET | `/api/v1/users/{player_id}` | `get_profile` |
| Users | PATCH | `/api/v1/users/{player_id}` | `update_profile` |
| Users | DELETE | `/api/v1/users/{player_id}` | `delete_player` |
| Stats | GET | `/api/v1/users/{player_id}/stats` | `get_stats` |
| Stats | PATCH | `/api/v1/users/{player_id}/stats` | `update_stats` |
| Stats | POST | `/api/v1/users/{player_id}/online` | `update_online` |
| Stats | POST | `/api/v1/users/{player_id}/join` | `update_join` |
| Shop | GET | `/api/v1/shop` | `list_items` |
| Shop | POST | `/api/v1/shop` | `create_item` |
| Shop | GET | `/api/v1/shop/{id}` | `get_item` |
| Shop | PATCH | `/api/v1/shop/{id}` | `update_item` |
| Shop | DELETE | `/api/v1/shop/{id}` | `delete_item` |
| Inventory | GET | `/api/v1/users/{player_id}/inventory` | `list_inventory` |
| Inventory | POST | `/api/v1/users/{player_id}/inventory` | `purchase_item` |
| Inventory | DELETE | `/api/v1/users/{player_id}/inventory/{item_id}` | `remove_item` |
| Inventory | GET | `/api/v1/users/{player_id}/inventory/has/{shop_item_id}` | `has_item` |
| Settings | GET | `/api/v1/users/{player_id}/settings` | `get_settings` |
| Settings | PATCH | `/api/v1/users/{player_id}/settings` | `update_settings` |
| Settings | PUT | `/api/v1/users/{player_id}/settings/{key}` | `update_single` |
| Settings | DELETE | `/api/v1/users/{player_id}/settings/{key}` | `delete_setting` |
| Leaderboard | GET | `/api/v1/leaderboard/{category}` | `get_leaderboard` |
| Leaderboard | GET | `/api/v1/leaderboard/{category}/{player_id}` | `get_player_rank` |

---

## Database Schema (6 tables)

| Table | Purpose | Key Features |
|-------|---------|--------------|
| `users` | Player accounts | Unique `player_id`, role ENUM (`guest`, `admin`, `owner`), soft delete |
| `user_stats` | Player statistics | Optimistic locking (`version`), `total_spent` denormalized |
| `shop_items` | Shop catalog | Game role ENUM (`survival`, `killer`, `both`), `discounted_price` computed |
| `user_inventory` | Player purchases | Unique `(user_id, shop_item_id)`, `equipped` flag |
| `user_settings` | Key-value settings | `TINYINT UNSIGNED` values (0-255) |
| `leaderboard` | Ranked scores | `DENSE_RANK()` window function, 4 categories |

> **Role Types:**
> - **User roles** (access control): `guest`, `admin`, `owner`
> - **Game roles** (shop items): `survival`, `killer`, `both`

---

## Where to Look

| I want to... | Look at... |
|--------------|-----------|
| Add an API endpoint | `src/routes.rs` + `src/handlers/` + `src/services/` |
| Add a database table | `migrations/` + `src/models/` |
| Modify error handling | `src/errors.rs` (AppError enum) |
| Change auth middleware | `src/middleware/apikey.rs` |
| Add request validation | Model structs in `src/models/` (use `validator` derive) |
| Modify JSON response format | `src/models/mod.rs` (response helpers) |
| Change pagination behavior | Service files in `src/services/` (cursor pattern) |
| Update Docker setup | `Dockerfile`, `docker-compose.yml`, `docker-compose.prod.yml` |
| Add tests | `tests/integration/` (currently skeletal) |
| Review design decisions | `docs/superpowers/specs/2026-09-16-roblox-backend-design.md` |
