use dfps_web_frontend::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}
