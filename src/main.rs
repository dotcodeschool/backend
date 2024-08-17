use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::info;
use utils::generate_repo_id;
mod utils;

/// Create a repository on the git server
#[post("/api/v0/create-repository")]
async fn create_repository(template: String) -> impl Responder {
    let repo_name = generate_repo_id();
    info!(
        "Created repository `{}` using template `{}`",
        repo_name, template
    );
    HttpResponse::Ok().body(format!(
        "Created repository `{}` using template `{}`",
        repo_name, template
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", &bind_address);
    HttpServer::new(|| App::new().service(create_repository))
        .bind(&bind_address)
        .expect(format!("Couldn't bind to {}", bind_address).as_str())
        .run()
        .await
}
