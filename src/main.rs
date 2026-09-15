mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use actix_web::{web, App, HttpResponse, HttpServer};
use config::Config;
use middleware::apikey::ApiKey;
use tracing_subscriber::fmt::init;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "status": "ok"
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let port = config.api_port;

    // Initialize database pool
    let pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            // Health check endpoint (unauthenticated)
            .route("/health", web::get().to(health_check))
            // API v1 routes (authenticated)
            .service(
                web::scope("/api/v1")
                    .wrap(ApiKey)
                    .configure(routes::configure),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
