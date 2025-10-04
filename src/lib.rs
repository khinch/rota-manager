use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    serve::Serve,
    Json, Router,
};

use redis::{Client, RedisResult};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error::Error;
use tokio::signal;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Level;

use domain::{AuthAPIError, ProjectAPIError};
pub mod routes;
use crate::utils::tracing::*;
use routes::{
    auth::{delete_user, login, logout, signup, verify_2fa, verify_token},
    projects::{
        add_member, add_shift, get_member, get_member_list_for_project,
        get_project, get_project_list, new_project, update_member,
    },
};
pub mod app_state;
pub mod domain;
pub mod services;
use app_state::AppState;
pub mod utils;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AuthAPIError::UserAlreadyExists => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::CONFLICT, "User already exists".to_string())
            }
            AuthAPIError::ValidationError(message) => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::BAD_REQUEST, format!("{message}"))
            }
            AuthAPIError::UserNotFound => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::NOT_FOUND, "User not found".to_string())
            }
            AuthAPIError::IncorrectCredentials => {
                log_error_chain(&self, Level::DEBUG);
                (
                    StatusCode::UNAUTHORIZED,
                    "Incorrect credentials".to_string(),
                )
            }
            AuthAPIError::UnexpectedError(_) => {
                log_error_chain(&self, Level::ERROR);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unexpected error".to_string(),
                )
            }
            AuthAPIError::MissingToken => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::BAD_REQUEST, "Missing token".to_string())
            }
            AuthAPIError::InvalidToken => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
        };
        let body = Json(ErrorResponse {
            error: error_message,
        });
        (status, body).into_response()
    }
}

impl IntoResponse for ProjectAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ProjectAPIError::IDNotFoundError(id) => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::NOT_FOUND, format!("{id}"))
            }
            ProjectAPIError::IDExistsError(id) => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::CONFLICT, format!("{id}"))
            }
            ProjectAPIError::AuthenticationError(auth_error) => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::UNAUTHORIZED, format!("{auth_error}"))
            }
            ProjectAPIError::UnexpectedError(_) => {
                log_error_chain(&self, Level::ERROR);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unexpected error".to_string(),
                )
            }
            ProjectAPIError::ValidationError(message) => {
                log_error_chain(&self, Level::DEBUG);
                (StatusCode::BAD_REQUEST, format!("{message}"))
            }
        };
        let body = Json(ErrorResponse {
            error: error_message,
        });
        (status, body).into_response()
    }
}

fn log_error_chain(e: &(dyn Error + 'static), debug_level: Level) {
    let separator =
        "\n-----------------------------------------------------------------------------------\n";
    let mut report = format!("{}{:?}\n", separator, e);
    let mut current = e.source();
    while let Some(cause) = current {
        let str = format!("Caused by:\n\n{:?}", cause);
        report = format!("{}\n{}", report, str);
        current = cause.source();
    }
    report = format!("{}\n{}", report, separator);
    match debug_level {
        Level::ERROR => tracing::error!("{}", report),
        Level::WARN => tracing::warn!("{}", report),
        Level::INFO => tracing::info!("{}", report),
        Level::DEBUG => tracing::debug!("{}", report),
        Level::TRACE => tracing::trace!("{}", report),
    }
}

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(
        app_state: AppState,
        address: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://localhost:3000".parse()?,
            "http://127.0.0.1:3000".parse()?,
            "https://rota-manager.testwebsitepleaseignore.uk:3000".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .route("/auth/signup", post(signup))
            .route("/auth/login", post(login))
            .route("/auth/verify-2fa", post(verify_2fa))
            .route("/auth/logout", post(logout))
            .route("/auth/verify-token", post(verify_token))
            .route("/auth/delete-user", delete(delete_user))
            .route("/projects/new", post(new_project))
            .route("/projects/list", get(get_project_list))
            .route("/projects/add-member", post(add_member))
            .route("/projects/get-members", get(get_member_list_for_project))
            .route("/projects/get-member", get(get_member))
            .route("/projects/update-member", put(update_member))
            .route("/projects/shifts", post(add_shift))
            .route("/projects/project", get(get_project))
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.with_graceful_shutdown(shutdown_signal()).await
    }
}

#[allow(dead_code)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn get_postgres_pool(
    url: &Secret<String>,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.expose_secret())
        .await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
