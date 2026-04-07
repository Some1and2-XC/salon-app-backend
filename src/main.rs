mod middleware;
mod models;
mod routes;

use dotenv::dotenv;
use reqwest::Client;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use utoipa::openapi::{Server, security::HttpBuilder};
use utoipa_axum::{router::OpenApiRouter, routes};
use std::{env, str::FromStr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use middleware::auth::FirebaseKeyCache;

use utoipa_swagger_ui::SwaggerUi;

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
    // sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        http_client: Client::new(),
        key_cache: FirebaseKeyCache::default(),
        firebase_project_id,
    };

    let (router, mut api) = build_router(state).split_for_parts();
    api.info.title = env!("CARGO_PKG_NAME").to_string();
    api.info.version = env!("CARGO_PKG_VERSION").to_string();
    api.info.description = Some("Rust/Axum backend for the appointment booking system, with Firebase Authentication.".to_string());
    let mut contact = utoipa::openapi::Contact::new();
    contact.name = Some("Mark Tobin".to_string());
    contact.email = Some("mr648072@dal.ca".to_string());
    contact.url = None;
    contact.extensions = None;
    api.info.contact = Some(contact);
    api.info.license = None;
    api.info.terms_of_service = None;
    api.servers = Some(vec![
        Server::new("/api"),
        // Server::new("/"),
    ]);

    if api.components.is_none() {
        api.components = Some(utoipa::openapi::Components::new());
    }
    api.components.as_mut().unwrap().add_security_scheme(
        "bearer_token",
        utoipa::openapi::security::SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        )
    );

    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        // .merge(Redoc::with_url("/redoc", api.clone()))
        .layer(CorsLayer::permissive()) // TODO fix this. This is bad.
        ;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Listening on http://0.0.0.0:{port}");
    tracing::info!("API Docs can be found at: http://0.0.0.0:{port}/swagger-ui");
    axum::serve(listener, router.into_make_service()).await?;
    // axum::serve(listener, router.into_make_service())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn build_router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        // ── Users ──────────────────────────────────────────────────────────
        .routes(routes!(routes::users::get_me))
        .routes(routes!(routes::users::update_me))
        .routes(routes!(routes::users::create_user))
        .routes(routes!(routes::users::get_user_by_id)) // admin
        // ── Tasks ──────────────────────────────────────────────────────────
        .routes(routes!(routes::tasks::list_tasks))
        .routes(routes!(routes::tasks::create_task))
        .routes(routes!(routes::tasks::get_task))
        .routes(routes!(routes::tasks::update_task))
        .routes(routes!(routes::tasks::delete_task))
        // ── Task Categories ────────────────────────────────────────────────
        .routes(routes!(routes::task_category::list_task_categories))
        // ── Employees ──────────────────────────────────────────────────────
        .routes(routes!(routes::employees::list_employees))
        .routes(routes!(routes::employees::create_employee))
        .routes(routes!(routes::employees::get_employee))
        .routes(routes!(routes::employees::update_employee))
        .routes(routes!(routes::employees::delete_employee))
        // ── Appointments ───────────────────────────────────────────────────
        .routes(routes!(routes::appointments::list_appointments))
        .routes(routes!(routes::appointments::create_appointment))
        .routes(routes!(routes::appointments::get_appointment))
        .routes(routes!(routes::appointments::update_appointment))
        .routes(routes!(routes::appointments::delete_appointment))
        // ── Availability ───────────────────────────────────────────────────
        .routes(routes!(routes::availability::list_availability))
        .routes(routes!(routes::availability::create_availability))
        .routes(routes!(routes::availability::delete_availability))
        // ── Middleware ─────────────────────────────────────────────────────
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())

        .with_state(state)
}
