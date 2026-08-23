//! The metw-accounts-center web API.

use app::app;
use state::Config;
use std::{env, net::SocketAddr};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    metw_observability::init_tracing("metw-accounts".into());

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
