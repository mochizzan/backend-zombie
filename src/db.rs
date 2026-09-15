use sqlx::mysql::MySqlPool;

pub type DbPool = MySqlPool;

pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    MySqlPool::connect(database_url).await
}
