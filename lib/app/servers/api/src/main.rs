use dfps_api::{ApiServerConfig, init_logging, run};
use dfps_configuration::load_env;

#[tokio::main]
async fn main() {
    if let Err(err) = load_env("app.web.api") {
        eprintln!("dfps_api env error: {err}");
        std::process::exit(1);
    }
    init_logging();
    if let Err(err) = run(ApiServerConfig::default()).await {
        eprintln!("dfps_api server error: {err}");
        std::process::exit(1);
    }
}
