//! The metw-accounts-center web API.

use app::app;
use state::Config;
use std::{env, net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let sock_addr: SocketAddr = env::var("HOST")
        .unwrap_or_else(|_| {
            tracing::info!("HOST environment variable is not set, defaulting to 127.0.0.1:3781");

            "127.0.0.1:3781".to_string()
        })
        .parse()
        .unwrap();

    let listener = tokio::net::TcpListener::bind(&sock_addr).await.unwrap();

    let config = Config::from_env();

    let state = config.bootstrap().await;

    #[cfg(debug_assertions)]
    let app = {
        use tower_http::cors::{AllowHeaders, AllowMethods, Any, CorsLayer};

        let cors = CorsLayer::new()
            .allow_methods(AllowMethods::any())
            .allow_origin(Any)
            .allow_headers(AllowHeaders::any());

        app(state).layer(cors)
    };
    #[cfg(not(debug_assertions))]
    let app = app(state);

    axum::serve(listener, app).await.unwrap();
}
