mod models;
mod utils;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::{error, info};
use models::Repository;
use mongodb::Client;
use utils::*;

const DB_NAME: &str = "dcs-test";
const COLLECTION_NAME: &str = "repositories";

#[derive(serde::Deserialize)]
struct CreateRepoRequest {
    template: String,
    user_id: String,
}

/// Create a repository on the git server
#[post("/api/v0/create-repository")]
async fn create_repository(
    client: web::Data<Client>,
    json: web::Json<CreateRepoRequest>,
) -> impl Responder {
    let repo_name = generate_repo_id();
    let collection = client.database(DB_NAME).collection(COLLECTION_NAME);

    info!(
        "Creating repository `{}` using template `{}`",
        repo_name, json.template
    );

    if let Err(e) = create_git_repo(&repo_name, &json.template).await {
        error!("Failed to create repository on git server: {}", e);
        return HttpResponse::InternalServerError()
            .body("Failed to create repository on git server");
    }

    info!(
        "Successfully created repository `{}` on git server",
        repo_name
    );

    let repository = Repository {
        name: repo_name.clone(),
        template: json.template.clone(),
        relationships: vec![models::Relationship {
            id: json.user_id.clone(),
            r#type: models::DocumentType::User,
        }],
    };

    if let Err(e) = collection.insert_one(repository).await {
        error!("Failed to insert repository into database: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save repository to database");
    }

    info!(
        "Successfully inserted repository `{}` into database",
        repo_name
    );
    HttpResponse::Ok().body(format!(
        "Created repository `{}` using template `{}`",
        repo_name, json.template
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(&uri)
        .await
        .expect("Failed to connect to MongoDB");

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    info!("Starting server on {}", &bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(create_repository)
    })
    .bind(&bind_address)
    .unwrap_or_else(|_| panic!("Failed to bind to {}", bind_address))
    .run()
    .await
}
