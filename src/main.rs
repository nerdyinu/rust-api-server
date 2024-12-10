use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;

mod db;
mod error;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("api_server=debug".parse().unwrap()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/items", get(handlers::items::list_items))
        .route("/items", post(handlers::items::create_item))
        .route("/items/:id", get(handlers::items::get_item))
        .route("/items/:id", put(handlers::items::update_item))
        .route("/items/:id", delete(handlers::items::delete_item))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
