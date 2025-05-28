mod api;
mod config;
mod core;
mod db;
mod error;
mod models;
mod services;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::core::news_verification::{AIModel, BlockchainClient, NewsVerificationService};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::load().expect("Failed to load configuration");

    // Initialize database connection
    let db_pool = db::init_pool(&config.database)
        .await
        .expect("Failed to initialize database pool");

    // Initialize Redis connection
    let redis_client = db::init_redis(&config.redis)
        .await
        .expect("Failed to initialize Redis client");

    // Initialize Kafka producer
    let kafka_producer = db::init_kafka_producer(&config.kafka)
        .expect("Failed to initialize Kafka producer");

    // Initialize news verification service
    let ai_model = AIModel::new("models/news_verification".to_string());
    let blockchain_client = BlockchainClient::new(
        "https://mainnet.infura.io/v3/your-project-id".to_string(),
        "0x123...".to_string(),
    );
    let news_verification_service = NewsVerificationService::new(ai_model, blockchain_client);

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/events", post(api::events::create_event))
        .route("/api/v1/analytics", get(api::analytics::get_analytics))
        // News verification routes
        .route("/api/v1/news/verify", post(api::news::verify_article))
        .route(
            "/api/v1/news/status/:article_id",
            get(api::news::get_verification_status),
        )
        .route(
            "/api/v1/news/proof/:article_id",
            get(api::news::get_blockchain_proof),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            db: db_pool,
            redis: redis_client,
            kafka: kafka_producer,
            news_verification: news_verification_service,
        });

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Application state
#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    redis: redis::Client,
    kafka: rdkafka::producer::FutureProducer,
    news_verification: NewsVerificationService,
}

// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
} 