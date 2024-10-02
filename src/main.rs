mod constants;
mod errors;
mod helpers;
mod models;
mod types;
mod utils;

use actix_web::{get, post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use helpers::{
	fetch_course_success_response, get_repository_success_response, handle_db_error,
	handle_repo_creation_error, repository_creation_success_response,
	submission_creation_success_response,
};
use log::info;
use mongodb::Client;
use types::*;
use utils::{do_create_repo, do_create_submission, fetch_course, get_repo_from_db};

#[get("/course/{course_id}")]
async fn get_course_v0(data: web::Data<AppState>, course_id: web::Path<String>) -> impl Responder {
	match fetch_course(&data.client, &course_id).await {
		Ok(course) => fetch_course_success_response(course),
		Err(e) => handle_db_error(e),
	}
}

/// Create a repository on the git server
#[post("/repository")]
async fn create_repository_v0(
	data: web::Data<AppState>,
	json: web::Json<CreateRepoRequest>,
) -> impl Responder {
	match do_create_repo(&data.client, &json).await {
		Ok(repo_name) => repository_creation_success_response(repo_name, &json.repo_template),
		Err(e) => handle_repo_creation_error(e),
	}
}

#[get("/repository/{repo_name}")]
async fn get_repository_v0(
	data: web::Data<AppState>,
	repo_name: web::Path<String>,
) -> impl Responder {
	match get_repo_from_db(&data.client, repo_name.as_str()).await {
		Ok(repository) => get_repository_success_response(repository),
		Err(e) => handle_db_error(e),
	}
}

#[post("/submission")]
async fn create_submission_v0(
	data: web::Data<AppState>,
	json: web::Json<CreateSubmissionRequest>,
) -> impl Responder {
	match do_create_submission(&data.client, &data.redis_uri, &data.ws_url, &json).await {
		Ok(submission_response) => submission_creation_success_response(submission_response),
		Err(e) => handle_db_error(e),
	}
}

pub struct AppState {
	client: Client,
	redis_uri: String,
	ws_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();
	dotenv().ok();

	let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
	let redis_uri = std::env::var("REDIS_URI").expect("REDIS_URI must be set");
	let ws_url = std::env::var("WS_URL").expect("WS_URL must be set");

	let client = Client::with_uri_str(&uri).await.expect("Failed to connect to MongoDB");

	let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
	let bind_address = format!("0.0.0.0:{}", port);

	info!("Starting server on {}", &bind_address);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(AppState {
				client: client.clone(),
				redis_uri: redis_uri.clone(),
				ws_url: ws_url.clone(),
			}))
			.service(
				web::scope("/api/v0")
					.service(create_repository_v0)
					.service(create_submission_v0)
					.service(get_course_v0)
					.service(get_repository_v0),
			)
	})
	.bind(&bind_address)
	.unwrap_or_else(|_| panic!("Failed to bind to {}", bind_address))
	.run()
	.await
}
