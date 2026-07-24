mod email_limiting_repo;

use super::Config;

// Bootstrap all services and clients, for testing .env
#[tokio::test]
#[ignore]
async fn state_from_env() {
    dotenvy::dotenv_override().unwrap();

    let config = Config::from_env();

    config.bootstrap().await;
}
