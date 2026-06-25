mod email_limiting_repo;

use super::Config;

// Test validity of the .env.example file
#[test]
#[serial_test::serial]
fn config_from_example_env() {
    dotenvy::from_path_override("../.env.example").unwrap();

    Config::from_env();
}

// Bootstrap all services and clients, for testing .env
#[tokio::test]
#[ignore]
#[serial_test::serial]
async fn state_from_env() {
    dotenvy::dotenv_override().unwrap();

    let config = Config::from_env();

    config.bootstrap().await;
}
