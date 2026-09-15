# Roblox Backend API — Endpoint Documentation

**Base URL:** `http://localhost:3000/api/v1`

**Auth:** Semua endpoint (kecuali `/health`) membutuhkan header `X-API-Key`.

---

## Table of Contents

1. [Common](#common)
2. [Users](#users)
3. [User Stats](#user-stats)
4. [Shop](#shop)
5. [Inventory](#inventory)
6. [Settings](#settings)
7. [Leaderboard](#leaderboard)

---

## Common

### Authentication

```
X-API-Key: <your-api-key>
```

Semua request ke `/api/v1/*` wajib menyertakan header ini. Tanpa header atau key salah → `401 UNAUTHORIZED`.

### Response Format

**Success (single):**
```json
{
  "success": true,
  "data": { ... }
}
```

**Success (list with pagination):**
```json
{
  "success": true,
  "data": [ ... ],
  "meta": {
    "cursor": "MTA=",  
    "has_more": true
  }
}
```

**Error:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "User 'xyz' not found"
  }
}
```

### Error Codes

| Code | HTTP Status | Keterangan |
|------|-------------|------------|
| `NOT_FOUND` | 404 | Resource tidak ditemukan |
| `VALIDATION_ERROR` | 400 | Input tidak valid |
| `CONFLICT` | 409 | Resource sudah ada (duplicate) |
| `VERSION_CONFLICT` | 409 | Optimistic lock version mismatch |
| `UNAUTHORIZED` | 401 | API key salah/tidak ada |
| `INTERNAL_ERROR` | 500 | Server error |
| `DATABASE_ERROR` | 500 | Database error |

### Cursor Pagination

Untuk endpoint list, gunakan query parameter:

| Parameter | Default | Max | Keterangan |
|-----------|---------|-----|------------|
| `cursor` | - | - | Token dari response sebelumnya (`meta.cursor`) |
| `limit` | 20 | 100 | Jumlah item per halaman |

Cara pakai:
1. Request pertama: `GET /api/v1/shop?limit=10`
2. Response punya `meta.cursor` dan `meta.has_more`
3. Jika `has_more: true`, request berikutnya: `GET /api/v1/shop?limit=10&cursor=<cursor_value>`

---

## Users

### POST /users — Register / Login

Register player baru atau update jika `player_id` sudah ada (upsert).

**Request Body:**
```json
{
  "player_id": "roblox_player_123",    // wajib, 1-64 char
  "username": "PlayerName",            // wajib, 1-64 char
  "nickname": "CoolNickname",          // wajib, 1-64 char
  "role": "guest"                      // opsional, "guest" | "admin" | "owner", default "guest"
}
```

> **User Roles:**
> - `guest` — Player biasa (default)
> - `admin` — Administrator
> - `owner` — Pemilik game

**Response: `201 Created`**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "player_id": "roblox_player_123",
    "username": "PlayerName",
    "nickname": "CoolNickname",
    "role": "guest",
    "created_at": "2026-09-16T01:00:00",
    "updated_at": "2026-09-16T01:00:00",
    "deleted_at": null
  }
}
```

---

### GET /users/{player_id} — Get Profile

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "player_id": "roblox_player_123",
    "username": "PlayerName",
    "nickname": "CoolNickname",
    "role": "guest",
    "created_at": "2026-09-16T01:00:00",
    "updated_at": "2026-09-16T01:00:00",
    "deleted_at": null
  }
}
```

---

### PATCH /users/{player_id} — Update Profile

Update field tertentu saja (yang dikirim akan di-update, yang tidak dikirim tetap sama).

**Request Body (semua field opsional):**
```json
{
  "username": "NewName",
  "nickname": "NewNickname",
  "role": "admin"
}
```

**Response: `200 OK`** — return user terbaru

---

### DELETE /users/{player_id} — Delete Player

Soft delete player dan semua data terkait (stats, inventory, settings).

**Response: `204 No Content`** (body kosong)

---

## User Stats

### GET /users/{player_id}/stats — Get Stats

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "user_id": 1,
    "exp": 15000,
    "fang": 500,
    "bcoin": 10000,
    "level": 25,
    "total_playtime": 36000,
    "matches_played": 150,
    "total_spent": 2500,
    "version": 3,
    "last_online_at": "2026-09-16T12:00:00",
    "last_join_at": "2026-09-16T10:00:00",
    "created_at": "2026-09-16T01:00:00",
    "updated_at": "2026-09-16T12:00:00"
  }
}
```

---

### PATCH /users/{player_id}/stats — Update Stats

**Optimistic Locking:** Kirim `version` saat update. Jika version tidak match → `409 VERSION_CONFLICT`.

**Request Body (semua field opsional):**
```json
{
  "exp": 16000,
  "fang": 550,
  "bcoin": 10500,
  "level": 26,
  "total_playtime": 36100,
  "matches_played": 151,
  "version": 3
}
```

**Response: `200 OK`** — return stats terbaru dengan `version` yang sudah increment

**Error:**
```json
{
  "success": false,
  "error": {
    "code": "VERSION_CONFLICT",
    "message": "Expected version 3, got 2"
  }
}
```

> **Catatan:** Setiap update stats juga otomatis update leaderboard.

---

### POST /users/{player_id}/online — Update Online

Update `last_online_at` ke waktu sekarang.

**Response: `200 OK`**
```json
{
  "success": true,
  "data": "ok"
}
```

---

### POST /users/{player_id}/join — Update Join

Update `last_join_at` ke waktu sekarang.

**Response: `200 OK`**
```json
{
  "success": true,
  "data": "ok"
}
```

---

## Shop

> **Catatan:** Role di shop adalah **game role** (`survival`, `killer`, `both`), berbeda dengan user role (`guest`, `admin`, `owner`). Game role menentukan item untuk mode permainan apa.

### GET /shop — List Shop Items

**Query Parameters:**

| Parameter | Tipe | Keterangan |
|-----------|------|------------|
| `role` | string | Filter: `"survival"`, `"killer"`, atau `"both"` |
| `category` | string | Filter berdasarkan kategori item |
| `cursor` | string | Cursor untuk pagination |
| `limit` | integer | Jumlah item (default: 20, max: 100) |

**Contoh:** `GET /api/v1/shop?role=survival&category=weapon&limit=10`

**Response: `200 OK`**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "role": "survival",
      "category": "weapon",
      "name": "Steel Sword",
      "description": "Basic sword for survivors",
      "price": 500,
      "discount_percent": 10,
      "discounted_price": 450,
      "is_active": true,
      "created_at": "2026-09-16T01:00:00",
      "updated_at": "2026-09-16T01:00:00",
      "deleted_at": null
    }
  ],
  "meta": {
    "cursor": "MQ==",
    "has_more": true
  }
}
```

---

### GET /shop/{id} — Get Shop Item

**Response: `200 OK`** — return single ShopItem

---

### POST /shop — Create Shop Item

**Request Body:**
```json
{
  "role": "survival",           // wajib: "survival" | "killer" | "both" (game role)
  "category": "weapon",         // wajib
  "name": "Steel Sword",        // wajib
  "description": "Basic sword", // opsional
  "price": 500,                 // wajib
  "discount_percent": 10        // opsional, default 0
}
```

> **Game Roles (Shop):**
> - `survival` — Item untuk mode Survival
> - `killer` — Item untuk mode Killer
> - `both` — Item untuk kedua mode

> `discounted_price` dihitung otomatis: `price × (100 - discount_percent) / 100`

**Response: `201 Created`** — return item yang baru dibuat

---

### PATCH /shop/{id} — Update Shop Item

**Request Body (semua field opsional):**
```json
{
  "role": "killer",
  "category": "armor",
  "name": "Iron Shield",
  "description": "Strong shield",
  "price": 800,
  "discount_percent": 5,
  "is_active": false
}
```

**Response: `200 OK`** — return item terbaru

---

### DELETE /shop/{id} — Delete Shop Item

Soft delete item dari shop.

**Response: `204 No Content`**

---

## Inventory

### GET /users/{player_id}/inventory — List Inventory

**Query Parameters:**

| Parameter | Tipe | Keterangan |
|-----------|------|------------|
| `category` | string | Filter berdasarkan kategori item |
| `cursor` | string | Cursor untuk pagination |
| `limit` | integer | Jumlah item (default: 20, max: 100) |

**Response: `200 OK`**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "user_id": 1,
      "shop_item_id": 5,
      "equipped": false,
      "created_at": "2026-09-16T08:00:00",
      "deleted_at": null
    }
  ],
  "meta": {
    "cursor": null,
    "has_more": false
  }
}
```

---

### POST /users/{player_id}/inventory — Purchase Item

Beli item dari shop. Item akan ditambahkan ke inventory player.

**Request Body:**
```json
{
  "shop_item_id": 5    // wajib, ID item yang ingin dibeli
}
```

**Validasi:**
- Player harus exist
- Shop item harus exist dan `is_active: true`
- Duplikat akan ditolak (unique constraint)

> Otomatis menambah `total_spent` di user_stats seharga `discounted_price`.

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "user_id": 1,
    "shop_item_id": 5,
    "equipped": false,
    "created_at": "2026-09-16T08:00:00",
    "deleted_at": null
  }
}
```

---

### DELETE /users/{player_id}/inventory/{item_id} — Remove Item

Hapus item dari inventory (soft delete).

> **Catatan:** `{item_id}` adalah `id` dari inventory, bukan `shop_item_id`.

**Response: `204 No Content`**

---

### GET /users/{player_id}/inventory/has/{shop_item_id} — Check Ownership

Cek apakah player memiliki item tertentu.

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "player_id": "roblox_player_123",
    "shop_item_id": 5,
    "owned": true
  }
}
```

---

## Settings

### GET /users/{player_id}/settings — Get All Settings

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "music_enabled": 1,
    "sfx_enabled": 1,
    "language": 0,
    "difficulty": 2
  }
}
```

> Value selalu `u8` (0-255). Key bebas string, value: `TINYINT UNSIGNED`.

---

### PATCH /users/{player_id}/settings — Bulk Update Settings

Update atau tambah beberapa setting sekaligus (upsert).

**Request Body:**
```json
{
  "settings": {
    "music_enabled": 0,
    "sfx_enabled": 1,
    "difficulty": 3
  }
}
```

**Response: `200 OK`** — return semua settings terbaru (termasuk yang tidak diupdate)

---

### PUT /users/{player_id}/settings/{key} — Update Single Setting

Update atau tambah satu setting (upsert).

**Request Body:**
```json
{
  "value": 1    // wajib, 0-255
}
```

**Response: `200 OK`**
```json
{
  "success": true,
  "data": "ok"
}
```

---

### DELETE /users/{player_id}/settings/{key} — Delete Setting

Hapus satu setting (soft delete).

**Response: `204 No Content`**

---

## Leaderboard

### GET /leaderboard/{category} — Get Leaderboard

Ambil peringkat top players berdasarkan kategori.

**Categories:** `level`, `spent`, `playtime`, `matches`

**Query Parameters:**

| Parameter | Tipe | Keterangan |
|-----------|------|------------|
| `limit` | integer | Jumlah player (default: 20, max: 100) |
| `role` | string | Filter: `"survival"` atau `"killer"` |

**Contoh:** `GET /api/v1/leaderboard/level?limit=10&role=survival`

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "category": "level",
    "leaderboard": [
      {
        "rank": 1,
        "user_id": 1,
        "player_id": "roblox_player_123",
        "username": "ProPlayer",
        "nickname": "TheLegend",
        "score": 99999
      },
      {
        "rank": 2,
        "user_id": 2,
        "player_id": "roblox_player_456",
        "username": "GoodPlayer",
        "nickname": "Challenger",
        "score": 85000
      }
    ]
  }
}
```

> Menggunakan `DENSE_RANK()` — peringkat tanpa gap (1, 1, 2, 3, bukan 1, 1, 3, 4).

---

### GET /leaderboard/{category}/{player_id} — Get Player Rank

Ambil peringkat player tertentu di kategori tertentu.

**Response: `200 OK`**
```json
{
  "success": true,
  "data": {
    "rank": 5,
    "score": 50000,
    "total_players": 1250
  }
}
```

---

## Quick Reference

| Method | Endpoint | Keterangan |
|--------|----------|------------|
| `POST` | `/users` | Register / Login (upsert) |
| `GET` | `/users/{player_id}` | Get profile |
| `PATCH` | `/users/{player_id}` | Update profile |
| `DELETE` | `/users/{player_id}` | Delete player |
| `GET` | `/users/{player_id}/stats` | Get stats |
| `PATCH` | `/users/{player_id}/stats` | Update stats (optimistic lock) |
| `POST` | `/users/{player_id}/online` | Update online timestamp |
| `POST` | `/users/{player_id}/join` | Update join timestamp |
| `GET` | `/shop` | List shop items |
| `POST` | `/shop` | Create shop item |
| `GET` | `/shop/{id}` | Get shop item |
| `PATCH` | `/shop/{id}` | Update shop item |
| `DELETE` | `/shop/{id}` | Delete shop item |
| `GET` | `/users/{player_id}/inventory` | List inventory |
| `POST` | `/users/{player_id}/inventory` | Purchase item |
| `DELETE` | `/users/{player_id}/inventory/{item_id}` | Remove item |
| `GET` | `/users/{player_id}/inventory/has/{shop_item_id}` | Check ownership |
| `GET` | `/users/{player_id}/settings` | Get all settings |
| `PATCH` | `/users/{player_id}/settings` | Bulk update settings |
| `PUT` | `/users/{player_id}/settings/{key}` | Update single setting |
| `DELETE` | `/users/{player_id}/settings/{key}` | Delete setting |
| `GET` | `/leaderboard/{category}` | Get leaderboard |
| `GET` | `/leaderboard/{category}/{player_id}` | Get player rank |
