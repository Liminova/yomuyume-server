#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub server_port: i32,
    pub sqlite_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
}

impl Config {
    pub fn init() -> Self {
        let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1".to_string());
        let server_port = std::env::var("SERVER_PORT").unwrap_or(3000.to_string());
        let sqlite_url = std::env::var("SQLITE_URL").expect("SQLITE_URL must be set.");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let jwt_expires_in = std::env::var("JWT_EXPIRES_IN").expect("JWT_EXPIRES_IN must be set.");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set.");

        Self {
            server_address,
            server_port: server_port.parse().unwrap_or(3000),
            sqlite_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse().unwrap(),
        }
    }
}
