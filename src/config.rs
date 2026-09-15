use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub api_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;
        
        let api_key = env::var("API_KEY")
            .map_err(|_| "API_KEY environment variable is required")?;
        
        let api_port = env::var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "API_PORT must be a valid port number")?;

        Ok(Config {
            database_url,
            api_key,
            api_port,
        })
    }
}
