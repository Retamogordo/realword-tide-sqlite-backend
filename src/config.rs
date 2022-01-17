#[derive(Debug, Clone)]
pub struct Config {
    pub database_url_prefix: String,
    pub database_url_path: String,
    pub database_file: String,
    pub secret: String,
    pub drop_database: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        Self {
            database_url_prefix: std::env::var("DATABASE_URL_PREFIX").expect("No DATABASE_URL_PREFIX environment variable found"),
            database_url_path: std::env::var("DATABASE_URL_PATH").expect("No DATABASE_URL_PATH environment variable found"),
            database_file: std::env::var("DATABASE_FILE").expect("No DATABASE_FILE environment variable found"),
            secret: std::env::var("SECRET").expect("No SECRET environment variable found"),
            drop_database: 0 != std::env::var("DROP_DATABASE")
                .ok()
                .and_then(|s| s.parse::<u32>().ok() )
                .unwrap_or(0) 
        }
    }
}
