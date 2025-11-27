use refractive_swan_api::{ApiConfig, init_logging, run};

#[tokio::main]
async fn main() {
    init_logging();
    let config = ApiConfig::from_env_or_exit();
    if let Err(err) = run(config).await {
        eprintln!("refractive_swan_api server error: {err}");
        std::process::exit(1);
    }
}
