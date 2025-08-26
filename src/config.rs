use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub db_max_connections: u32,
    pub jwt_expiration_days: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set".to_string())?,
            jwt_secret: env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set".to_string())?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| "Invalid SERVER_PORT format".to_string())?,
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| "Invalid DB_MAX_CONNECTIONS format".to_string())?,
            jwt_expiration_days: env::var("JWT_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .map_err(|_| "Invalid JWT_EXPIRATION_DAYS format".to_string())?,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
