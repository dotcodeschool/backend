mod constants;
mod errors;
mod helpers;
mod models;
mod types;
mod utils;

use actix_web::{post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use helpers::{
	handle_repo_creation_error, handle_submission_creation_error,
	repository_creation_success_response, submission_creation_success_response,
};
use log::info;
use mongodb::Client;
use types::*;
use utils::{do_create_repo, do_create_submission};

/// Create a repository on the git server
#[post("/api/v0/create-repository")]
async fn create_repository(
	data: web::Data<AppState>,
	json: web::Json<CreateRepoRequest>,
) -> impl Responder {
	match do_create_repo(&data.client, &json).await {
		Ok(repo_name) => repository_creation_success_response(repo_name, &json.repo_template),
		Err(e) => handle_repo_creation_error(e),
	}
}

#[post("/api/v0/create-submission")]
async fn create_submission(
	data: web::Data<AppState>,
	json: web::Json<CreateSubmissionRequest>,
) -> impl Responder {
	match do_create_submission(&data.client, &data.redis_uri, &json).await {
		Ok(submission_response) => submission_creation_success_response(submission_response),
		Err(e) => handle_submission_creation_error(e),
	}
}

pub struct AppState {
	client: Client,
	redis_uri: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();
	dotenv().ok();

	let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
	let redis_uri = std::env::var("REDIS_URI").expect("REDIS_URI must be set");

	let client = Client::with_uri_str(&uri).await.expect("Failed to connect to MongoDB");

	let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
	let bind_address = format!("0.0.0.0:{}", port);

	info!("Starting server on {}", &bind_address);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(AppState {
				client: client.clone(),
				redis_uri: redis_uri.clone(),
			}))
			.service(create_repository)
			.service(create_submission)
	})
	.bind(&bind_address)
	.unwrap_or_else(|_| panic!("Failed to bind to {}", bind_address))
	.run()
	.await
}
