use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub storage_paths: Vec<String>,
    pub port: u16,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/localdrive".to_string());
        
        let storage_paths_str = env::var("STORAGE_PATHS")
            .unwrap_or_else(|_| "./storage".to_string());
        
        let storage_paths: Vec<String> = storage_paths_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .unwrap_or(3001);
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());
        
        Ok(Config {
            database_url,
            storage_paths,
            port,
            jwt_secret,
        })
    }
}