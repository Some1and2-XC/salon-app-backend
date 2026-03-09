mod middleware;
mod models;
mod routes;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use dotenv::dotenv;
use reqwest::Client;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{env, str::FromStr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use middleware::auth::FirebaseKeyCache;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(
    // // ── Users ──────────────────────────────────────────────────────────
    routes::users::get_me,
    // routes::users::update_me,
    // routes::users::create_user,
    // routes::users::get_user_by_id, // adm
    // // ── Tasks ──────────────────────────────────────────────────────────
    // routes::tasks::list_tasks,
    // routes::tasks::create_task,
    // routes::tasks::get_task,
    // routes::tasks::update_task,
    // routes::tasks::delete_task,
    // // ── Employees ──────────────────────────────────────────────────────
    // routes::employees::list_employees,
    // routes::employees::create_employee,
    // routes::employees::get_employee,
    // routes::employees::update_employee,
    // routes::employees::delete_employee,
    // // ── Appointments ───────────────────────────────────────────────────
    // routes::appointments::list_appointments,
    // routes::appointments::create_appointment,
    // routes::appointments::get_appointment,
    // routes::appointments::update_appointment,
    // routes::appointments::delete_appointment,
    // // ── Availability ───────────────────────────────────────────────────
    // routes::availability::list_availability,
    // routes::availability::create_availability,
    // routes::availability::delete_availability,
))]
struct APIDocs;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    // pub db: sqlx::PgPool,
    pub db: sqlx::SqlitePool,
    pub http_client: Client,
    pub key_cache: FirebaseKeyCache,
    pub firebase_project_id: String,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "appointment_backend=debug,tower_http=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let firebase_project_id =
        env::var("FIREBASE_PROJECT_ID").expect("FIREBASE_PROJECT_ID must be set");
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());

    let connect_opts = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        ;

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _| Box::pin(async move {
            sqlx::query("PRAGMA foreign_keys = ON;")
                .execute(conn)
                .await?;
            Ok(())
        }))
        .connect_with(connect_opts)
        // .connect(&database_url)
        .await?
        ;

    // Run pending migrations on startup.
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        http_client: Client::new(),
        key_cache: FirebaseKeyCache::default(),
        firebase_project_id,
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn build_router(state: AppState) -> Router {
    Router::new()
        // .merge(SwaggerUi::new("/swagger-ui")
        //     .url("/api-docs/openapi.json", APIDocs::openapi())
        // )
        // ── Users ──────────────────────────────────────────────────────────
        .route("/users/me", get(routes::users::get_me))
        .route("/users/me", patch(routes::users::update_me))
        .route("/users", post(routes::users::create_user))
        .route("/users/:uuid", get(routes::users::get_user_by_id)) // admin
        // ── Tasks ──────────────────────────────────────────────────────────
        .route("/tasks", get(routes::tasks::list_tasks))
        .route("/tasks", post(routes::tasks::create_task))
        .route("/tasks/:id", get(routes::tasks::get_task))
        .route("/tasks/:id", patch(routes::tasks::update_task))
        .route("/tasks/:id", delete(routes::tasks::delete_task))
        // ── Employees ──────────────────────────────────────────────────────
        .route("/employees", get(routes::employees::list_employees))
        .route("/employees", post(routes::employees::create_employee))
        .route("/employees/:id", get(routes::employees::get_employee))
        .route("/employees/:id", patch(routes::employees::update_employee))
        .route("/employees/:id", delete(routes::employees::delete_employee))
        // ── Appointments ───────────────────────────────────────────────────
        .route("/appointments", get(routes::appointments::list_appointments))
        .route("/appointments", post(routes::appointments::create_appointment))
        .route("/appointments/:uuid", get(routes::appointments::get_appointment))
        .route("/appointments/:uuid", patch(routes::appointments::update_appointment))
        .route("/appointments/:uuid", delete(routes::appointments::delete_appointment))
        // ── Availability ───────────────────────────────────────────────────
        .route("/availability", get(routes::availability::list_availability))
        .route("/availability", post(routes::availability::create_availability))
        .route("/availability/:id", delete(routes::availability::delete_availability))
        // ── Middleware ─────────────────────────────────────────────────────
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
