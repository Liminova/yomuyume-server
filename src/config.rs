#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,

    pub jwt_secret: String,
    pub jwt_maxage_hour: i64,

    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
}

impl Config {
    fn get_env(key: &str, default: Option<&str>) -> String {
        match default {
            Some(val) => std::env::var(key).unwrap_or(val.to_string()),
            None => std::env::var(key).unwrap_or_else(|_| panic!("{} must be set.", key)),
        }
    }

    fn may_get(key: &str, default: Option<&str>) -> Option<String> {
        match default {
            Some(val) => std::env::var(key).ok().or(Some(val.to_string())),
            None => None,
        }
    }

    pub fn init() -> Self {
        let app_name = Self::get_env("APP_NAME", Some("Yomuyume"));
        let server_address = Self::get_env("SERVER_ADDRESS", Some("127.0.0.1"));
        let server_port = Self::get_env("SERVER_PORT", Some("3000"))
            .parse()
            .unwrap_or(3000);
        let database_url = Self::get_env("DATABASE_URL", Some("sqlite:./database/sqlite.db"));

        let jwt_secret = Self::get_env("JWT_SECRET", None);
        let jwt_maxage_hour = Self::get_env("JWT_MAXAGE_HOUR", None)
            .parse()
            .unwrap_or(86400);

        let smtp_host = Self::may_get("SMTP_HOST", None);
        let smtp_port = Self::get_env("SMTP_PORT", Some("587"))
            .parse()
            .unwrap_or(587);
        let smtp_username = Self::get_env("SMTP_USERNAME", Some(""));
        let smtp_password = Self::get_env("SMTP_PASSWORD", Some(""));
        let smtp_from_email = Self::get_env("SMTP_FROM_EMAIL", Some(""));
        let smtp_from_name = Self::get_env("SMTP_FROM_NAME", Some(""));

        Self {
            app_name,
            server_address,
            server_port,
            database_url,

            jwt_secret,
            jwt_maxage_hour,

            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
        }
    }
}
