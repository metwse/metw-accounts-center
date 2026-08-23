//! The metw-accounts-center web API.

use app::app;
use state::Config;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env();

    let state = config.bootstrap().await;

    let app = app(state);

    metw_server::serve(app, "metw-accounts".into()).await;
}
