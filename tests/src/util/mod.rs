mod test_state;

pub use test_state::TestState;

/// Creates a PostgreSQL connection pool from `DATABASE_URL` enviroment
/// variable.
pub async fn pg_pool_from_env() -> sqlx::PgPool {
    dotenvy::dotenv_override().ok();

    sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}

/// Initializes a Redis client from `REDIS_URL` enviroment variable.
pub async fn redis_client_from_env() -> redis::aio::MultiplexedConnection {
    dotenvy::dotenv_override().ok();

    redis::Client::open(std::env::var("REDIS_URL").unwrap())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}
