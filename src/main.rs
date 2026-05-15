use metw_accounts_center::{App, state::Config};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::default())
        .init();

    dotenvy::dotenv().ok();

    sqlx::migrate!();

    let config = Config::from_env();

    App::bootstrap(config).await.serve().await;
}
