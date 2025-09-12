use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: Secret<String> = set_token();
    pub static ref APP_SERVICE_EXTERNAL_ADDRESS: String = load_or_default(
        "APP_SERVICE_EXTERNAL_ADDRESS",
        "http://localhost:8000"
    );
    pub static ref APP_SERVICE_CONTAINER_ADDRESS: String = load_or_default(
        "APP_SERVICE_CONTAINER_ADDRESS",
        "http://localhost:8000"
    );
    pub static ref DATABASE_URL: Secret<String> = get_db_url();
    pub static ref POSTMARK_AUTH_TOKEN: Secret<String> =
        set_postmark_auth_token();
    pub static ref POSTMARK_EMAIL_SENDER_ADDRESS: Secret<String> =
        set_postmark_email_sender_address();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
}

fn load_env() {
    dotenv().ok();
}

fn set_postmark_auth_token() -> Secret<String> {
    load_env();
    Secret::new(
        std_env::var(env::POSTMARK_AUTH_TOKEN_ENV_VAR)
            .expect("POSTMARK_AUTH_TOKEN must be set"),
    )
}

fn set_postmark_email_sender_address() -> Secret<String> {
    dotenv().ok();
    Secret::new(
        std_env::var(env::POSTMARK_EMAIL_SENDER_ADDRESS_ENV_VAR)
            .expect("POSTMARK_EMAIL_SENDER_ADDRESS must be set"),
    )
}

fn set_token() -> Secret<String> {
    load_env();
    let secret =
        std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    Secret::new(secret)
}

fn get_db_url() -> Secret<String> {
    load_env();
    let db_url =
        std_env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    if db_url.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }
    Secret::new(db_url)
}

fn load_or_default(variable_name: &str, default_value: &str) -> String {
    load_env();

    match std_env::var(variable_name) {
        Ok(value) => {
            if value.is_empty() {
                String::from(default_value)
            } else {
                value
            }
        }
        Err(_) => String::from(default_value),
    }
}

fn set_redis_host() -> String {
    load_env();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR)
        .unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

pub mod env {
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const POSTMARK_AUTH_TOKEN_ENV_VAR: &str = "POSTMARK_AUTH_TOKEN";
    pub const POSTMARK_EMAIL_SENDER_ADDRESS_ENV_VAR: &str =
        "POSTMARK_EMAIL_SENDER_ADDRESS";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/email";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
    pub mod email_client {
        use std::time::Duration;

        // pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}
