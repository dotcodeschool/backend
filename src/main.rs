mod models;
mod utils;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::{error, info};
use models::Repository;
use mongodb::Client;
use utils::generate_repo_id;

const DB_NAME: &str = "dcs-test";
const COLLECTION_NAME: &str = "repositories";
const TEMPLATE: &str = "rust-state-machine";

#[derive(serde::Deserialize)]
struct CreateRepoRequest {
    template: String,
    user_id: String,
}

/// Create a repository on the git server
#[post("/api/v0/create-repository")]
async fn create_repository(
    client: web::Data<Client>,
    form: web::Form<CreateRepoRequest>,
) -> impl Responder {
    let repo_name = generate_repo_id();
    let collection = client.database(DB_NAME).collection(COLLECTION_NAME);
    info!(
        "Creating repository `{}` using template `{}`",
        repo_name,
        form.template.clone()
    );
    let repository = Repository {
        name: repo_name.clone(),
        template: form.template.clone(),
        relationships: vec![models::Relationship {
            id: form.user_id.clone(),
            r#type: models::DocumentType::User,
        }],
    };
    
    let result = collection.insert_one(repository).await;
    match result {
        Ok(_) => {
            info!(
                "Created repository `{}` using template `{}`",
                repo_name, TEMPLATE
            );
            HttpResponse::Ok().body(format!(
                "Created repository `{}` using template `{}`",
                repo_name, TEMPLATE
            ))
        }
        Err(e) => {
            error!("Failed to create repository: {}", e);
            HttpResponse::InternalServerError().body("Failed to create repository")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(&uri)
        .await
        .expect("Couldn't connect to MongoDB");
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", &bind_address);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(create_repository)
    })
    .bind(&bind_address)
    .expect(format!("Couldn't bind to {}", bind_address).as_str())
    .run()
    .await
}
