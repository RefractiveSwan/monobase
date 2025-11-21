use refractive_swan_api::{ApiConfig, init_logging, run};

#[tokio::main]
async fn main() {
    init_logging();
    let config = match ApiConfig::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("refractive_swan_api config error: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = run(config).await {
        eprintln!("refractive_swan_api server error: {err}");
        std::process::exit(1);
    }
}
