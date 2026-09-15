use actix_web::web;

use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Users
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(handlers::user::register))
            .route("/{player_id}", web::get().to(handlers::user::get_profile))
            .route("/{player_id}", web::patch().to(handlers::user::update_profile))
            .route("/{player_id}", web::delete().to(handlers::user::delete_player))
            // User Stats
            .route("/{player_id}/stats", web::get().to(handlers::user_stats::get_stats))
            .route("/{player_id}/stats", web::patch().to(handlers::user_stats::update_stats))
            .route("/{player_id}/online", web::post().to(handlers::user_stats::update_online))
            .route("/{player_id}/join", web::post().to(handlers::user_stats::update_join))
            // Inventory
            .route("/{player_id}/inventory", web::get().to(handlers::inventory::list_inventory))
            .route("/{player_id}/inventory", web::post().to(handlers::inventory::purchase_item))
            .route("/{player_id}/inventory/{item_id}", web::delete().to(handlers::inventory::remove_item))
            .route("/{player_id}/inventory/has/{shop_item_id}", web::get().to(handlers::inventory::has_item))
            // Settings
            .route("/{player_id}/settings", web::get().to(handlers::settings::get_settings))
            .route("/{player_id}/settings", web::patch().to(handlers::settings::update_settings))
            .route("/{player_id}/settings/{key}", web::put().to(handlers::settings::update_single))
            .route("/{player_id}/settings/{key}", web::delete().to(handlers::settings::delete_setting)),
    );
    // Shop
    cfg.service(
        web::scope("/shop")
            .route("", web::get().to(handlers::shop::list_items))
            .route("", web::post().to(handlers::shop::create_item))
            .route("/{id}", web::get().to(handlers::shop::get_item))
            .route("/{id}", web::patch().to(handlers::shop::update_item))
            .route("/{id}", web::delete().to(handlers::shop::delete_item)),
    );
    // Leaderboard
    cfg.service(
        web::scope("/leaderboard")
            .route("/{category}", web::get().to(handlers::leaderboard::get_leaderboard))
            .route("/{category}/{player_id}", web::get().to(handlers::leaderboard::get_player_rank)),
    );
}
