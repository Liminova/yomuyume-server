#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i64,
}

impl Config {
    fn get_env(key: &str, default: Option<&str>) -> String {
        match default {
            Some(val) => std::env::var(key).unwrap_or(val.to_string()),
            None => std::env::var(key).unwrap_or_else(|_| panic!("{} must be set.", key)),
        }
    }

    pub fn init() -> Self {
        let server_address = Self::get_env("SERVER_ADDRESS", Some("127.0.0.1"));
        let server_port = Self::get_env("SERVER_PORT", Some("3000"))
            .parse()
            .unwrap_or(3000);
        let database_url = Self::get_env("DATABASE_URL", Some("sqlite:./database/sqlite.db"));
        let jwt_secret = Self::get_env("JWT_SECRET", None);
        let jwt_expires_in = Self::get_env("JWT_EXPIRES_IN", None);
        let jwt_maxage = Self::get_env("JWT_MAXAGE", None).parse().unwrap();

        Self {
            server_address,
            server_port,
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage,
        }
    }
}
