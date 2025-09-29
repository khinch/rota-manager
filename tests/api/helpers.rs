use reqwest::{cookie::Jar, Client, Response, StatusCode};
use rota_manager::{
    app_state::{
        AppState, BannedTokenStoreType, ProjectStoreType, TwoFACodeStoreType,
        UserStoreType,
    },
    domain::Email,
    get_postgres_pool, get_redis_client,
    services::{
        data_stores::{
            PostgresProjectStore, PostgresUserStore, RedisBannedTokenStore,
            RedisTwoFACodeStore,
        },
        postmark_email_client::PostmarkEmailClient,
    },
    utils::constants::{
        test, DATABASE_URL, POSTMARK_EMAIL_SENDER_ADDRESS, REDIS_HOST_NAME,
    },
    Application,
};
use secrecy::{ExposeSecret, Secret};
use serde_json::Value;
use sqlx::{
    postgres::{PgConnectOptions, PgConnection, PgPoolOptions},
    Connection, Executor, PgPool,
};
use std::{str::FromStr, sync::Arc};
use test_context::AsyncTestContext;
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::{
    matchers::method, matchers::path, Mock, MockServer, ResponseTemplate,
};

pub struct TestApp {
    pub address: String,
    pub banned_token_store: BannedTokenStoreType,
    pub cookie_jar: Arc<Jar>,
    pub email_server: MockServer,
    pub http_client: reqwest::Client,
    pub tmp_db_name: String,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub user_store: UserStoreType,
    pub project_store: ProjectStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        let tmp_db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&tmp_db_name).await;
        let user_store =
            Arc::new(RwLock::new(PostgresUserStore::new(pg_pool.clone())));
        let project_store =
            Arc::new(RwLock::new(PostgresProjectStore::new(pg_pool)));

        let redis_connection = Arc::new(RwLock::new(configure_redis()));
        let banned_token_store = Arc::new(RwLock::new(
            RedisBannedTokenStore::new(redis_connection.clone()),
        ));

        let two_fa_code_store =
            Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));

        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client = Arc::new(configure_postmark_email_client(base_url));

        let app_state = AppState::new(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client,
            project_store.clone(),
        );

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");
        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            banned_token_store,
            cookie_jar,
            email_server,
            http_client,
            tmp_db_name,
            two_fa_code_store,
            user_store,
            project_store,
        }
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/auth/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/auth/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/auth/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/auth/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_token<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/auth/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn delete_user(&self) -> reqwest::Response {
        self.http_client
            .delete(format!("{}/auth/delete-user", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_projects_new<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/projects/new", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_projects_list(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/projects/list", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_add_member<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/projects/add-member", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_member(&self, member_id: &str) -> reqwest::Response {
        self.http_client
            .get(format!("{}/projects/get-member", &self.address))
            .query(&[("memberId", member_id)])
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_members(&self, project_id: &str) -> reqwest::Response {
        self.http_client
            .get(format!("{}/projects/get-members", &self.address))
            .query(&[("projectId", project_id)])
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn put_member<Body>(
        &self,
        member_id: &str,
        body: &Body,
    ) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .put(format!("{}/projects/update-member", &self.address))
            .json(body)
            .query(&[("memberId", member_id)])
            .send()
            .await
            .expect("Failed to execute request")
    }
}

impl AsyncTestContext for TestApp {
    async fn setup() -> TestApp {
        TestApp::new().await
    }

    async fn teardown(self) {
        delete_database(&self.tmp_db_name).await;
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    configure_database(&postgresql_conn_url, db_name).await;

    let postgresql_conn_url_with_db = Secret::new(format!(
        "{}/{}",
        postgresql_conn_url.expose_secret(),
        db_name
    ));

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &Secret<String>, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string.expose_secret())
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string =
        format!("{}/{}", db_conn_string.expose_secret(), db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.expose_secret().to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender =
        Email::parse(POSTMARK_EMAIL_SENDER_ADDRESS.to_owned()).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}

pub async fn signup(
    app: &mut TestApp,
    email: &str,
    password: &str,
    two_fa: bool,
) {
    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": two_fa
        }))
        .await
        .status()
        .as_u16(),
        201 // TODO return some useful info here
    );
}

pub async fn login(app: &mut TestApp, email: &str, password: &str) {
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    match app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await
        .status()
        .as_u16()
    {
        x if x == 200 => (),
        x if x == 206 => {
            let two_fa_details = get_expected_2fa_details(app, email).await;
            verify_2fa(app, email, &two_fa_details.0, &two_fa_details.1).await;
        }
        e => panic!("Failed to log in. Expected 200 or 206, but got {e}. email: {email}, password: {password}"),
    }
}

pub async fn verify_2fa(app: &mut TestApp, email: &str, id: &str, code: &str) {
    assert_eq!(
        app.post_verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": id,
            "2FACode": code
        }))
        .await
        .status()
        .as_u16(),
        200 // TODO return some useful info here
    );
}

pub async fn get_expected_2fa_details(
    app: &mut TestApp,
    email: &str,
) -> (String, String) {
    let email = Email::parse(Secret::new(String::from(email)))
        .expect("Failed to parse email");

    let (expected_id, expected_two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("Failed to get 2FA data from store");

    (
        expected_id.as_ref().expose_secret().to_owned(),
        expected_two_fa_code.as_ref().expose_secret().to_owned(),
    )
}

pub async fn get_session(app: &mut TestApp, two_fa: bool) -> String {
    let email = get_random_email();
    let password = "password";

    signup(app, &email, &password, two_fa).await;
    login(app, &email, &password).await;

    email
}

pub async fn add_new_project(app: &mut TestApp, name: &str) -> String {
    let response = app
        .post_projects_new(&serde_json::json!({
            "name": name
        }))
        .await;

    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed to add new project with name: {name}"
    );

    let response_body: serde_json::Value =
        response.json().await.expect("Failed to parse JSON");

    let response_id = response_body
        .get("id")
        .expect("No ID in response")
        .as_str()
        .unwrap()
        .to_owned();

    response_id
}

pub async fn get_json_response_body(response: Response) -> Value {
    let body: Value = response
        .json()
        .await
        .expect("failed to parse response body JSON");
    body
}

pub async fn logout(app: &mut TestApp) {
    assert_eq!(
        app.post_logout().await.status().as_u16(),
        200,
        "Failed to log out"
    );
}

pub async fn delete_user(app: &mut TestApp) {
    assert_eq!(
        app.delete_user().await.status().as_u16(),
        200,
        "Failed to delete user"
    );
}

pub async fn add_member(
    app: &mut TestApp,
    name: &str,
    project_id: &str,
) -> String {
    let response = app
        .post_add_member(&serde_json::json!({
            "memberName": name,
            "projectId": project_id
        }))
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = get_json_response_body(response).await;
    body.get("memberId")
        .expect("Failed to read memberId from JSON response")
        .as_str()
        .expect("Failed to create str from memberId field")
        .to_owned()
}
