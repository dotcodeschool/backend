use std::collections::HashMap;

use log::{error, info, warn};
use mongodb::{bson::doc, Client};
use rand::prelude::*;

use crate::{
	constants::{DB_NAME, GIT_SERVER_URL, REPO_COLLECTION, SUBMISSION_COLLECTION, USER_COLLECTION},
	errors::{RepoCreationError, SubmissionCreationError},
	models,
	models::Repository,
	types::{CreateRepoRequest, CreateSubmissionRequest, CreateSubmissionResponse, DocumentType},
};

/// Generate a repository ID
/// This is used to identify the repository on the git server
pub(super) fn generate_repo_id() -> String {
	// Generate a random 16 character string
	let mut repo_id = hex::encode(rand::thread_rng().gen::<[u8; 16]>()).to_string();
	repo_id.truncate(16);
	repo_id
}

/// Generate a unique submission ID
pub(super) fn generate_submission_id() -> String {
	uuid::Uuid::new_v4().to_string()
}

/// Create a repository on the git server and insert it into the database
pub(super) async fn do_create_repo(
	client: &Client,
	json: &CreateRepoRequest,
) -> Result<String, RepoCreationError> {
	let repo_name = generate_repo_id();
	let repo_template = json.repo_template.clone();
	let user_id = json.user_id.clone();

	info!("Creating repository `{}` using template `{}`", repo_name, &repo_template);

	create_git_repo(&repo_name, &repo_template).await?;
	insert_repo_into_db(client, &repo_name, &repo_template, &user_id).await?;
	update_user_repo_list(client, &user_id, &repo_name).await?;

	info!("Successfully created repository `{}` on git server", repo_name);

	Ok(repo_name)
}

/// Send a post request to the git server to create a repository
async fn create_git_repo(repo_name: &str, template: &str) -> Result<(), RepoCreationError> {
	let client = reqwest::Client::new();
	let url = format!("{}/api/v0/create_repository", GIT_SERVER_URL);

	let json = HashMap::from([("repo_name", repo_name), ("template_repo", template)]);

	let request = client.post(&url);
	let request = add_bearer_token_if_available(request);

	let response = request.json(&json).send().await?;
	response.error_for_status()?;

	Ok(())
}

/// Insert a repository into the database
pub(super) async fn insert_repo_into_db(
	client: &Client,
	repo_name: &str,
	template: &str,
	user_id: &str,
) -> Result<(), RepoCreationError> {
	let collection = client.database(DB_NAME).collection(REPO_COLLECTION);

	let repository = Repository {
		repo_name: repo_name.to_string(),
		repo_template: template.to_string(),
		// TODO: Use the correct URL based on the template
		tester_url: "https://github.com/dotcodeschool/rust-state-machine-tester".to_string(),
		relationships: vec![models::Relationship {
			id: user_id.to_string(),
			r#type: DocumentType::User,
		}],
	};

	collection
		.insert_one(repository)
		.await
		.map(|_| info!("Successfully inserted repository `{}` into database", repo_name))
		.map_err(RepoCreationError::from)
}

/// Update the user's repository list in the database
pub(super) async fn update_user_repo_list(
	client: &Client,
	user_id: &str,
	repo_name: &str,
) -> Result<(), RepoCreationError> {
	let collection: mongodb::Collection<models::User> =
		client.database(DB_NAME).collection(USER_COLLECTION);

	let filter = doc! { "id": user_id };
	let update = doc! { "$addToSet": { "repositories": repo_name } };

	collection
		.update_one(filter, update)
		.await
		.map(|_| info!("Successfully updated user `{}` with repository `{}`", user_id, repo_name))
		.map_err(RepoCreationError::from)
}

/// Add a bearer token to the request if available
fn add_bearer_token_if_available(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
	match std::env::var("BEARER_TOKEN_SECRET") {
		Ok(token) => {
			info!("Using bearer token to authenticate with git server");
			request.bearer_auth(token)
		},
		Err(_) => {
			warn!("No bearer token found, proceeding without authentication");
			request
		},
	}
}

/// Create a submission for a repository.
/// This will generate a unique submission ID and return the logstream and tester URL.
/// The submission will be inserted into the database.
pub(super) async fn do_create_submission(
	client: &Client,
	redis_uri: &str,
	json: &CreateSubmissionRequest,
) -> Result<CreateSubmissionResponse, SubmissionCreationError> {
	let repo_name = &json.repo_name;
	let commit_sha = &json.commit_sha;

	info!("Creating submission for repository `{}` with commit `{}`", repo_name, commit_sha);

	let repository = get_repo_from_db(client, repo_name).await?;
	let tester_url = repository.tester_url.clone();

	let logstream_id = generate_submission_id();
	let logstream_url = format!("{}/{}", redis_uri, logstream_id);

	insert_submission_into_db(
		client,
		repo_name.to_string(),
		commit_sha.to_string(),
		logstream_id.to_string(),
		logstream_url.clone(),
	)
	.await?;

	info!(
		"Successfully created submission for repository `{}` with logstream url `{}`",
		repo_name, logstream_url
	);

	Ok(CreateSubmissionResponse { logstream_url, tester_url })
}

/// Fetch a repository from the database. Fail if the repository does not exist.
async fn get_repo_from_db(
	client: &Client,
	repo_name: &str,
) -> Result<Repository, SubmissionCreationError> {
	let collection = client.database(DB_NAME).collection(REPO_COLLECTION);

	let filter = doc! { "repo_name": repo_name };
	info!("Fetching repository `{}` from database", repo_name);
	let repository = collection.find_one(filter).await;

	info!("Repository: {:?}", repository);

	match repository {
		Ok(Some(repo)) => {
			info!("Successfully fetched repository `{}` from database", repo_name);
			Ok(repo)
		},
		Ok(None) => {
			error!("Repository `{}` not found in database", repo_name);
			Err(SubmissionCreationError::NotFound(actix_web::error::ErrorNotFound(format!(
				"Repository `{}` not found",
				repo_name
			))))
		},
		Err(e) => {
			error!("Error fetching repository `{}` from database: {:?}", repo_name, e);
			Err(SubmissionCreationError::DatabaseError(e))
		},
	}
}

/// Insert a submission into the database
async fn insert_submission_into_db(
	client: &Client,
	repo_name: String,
	commit_sha: String,
	logstream_id: String,
	logstream_url: String,
) -> Result<(), SubmissionCreationError> {
	let collection = client.database(DB_NAME).collection(SUBMISSION_COLLECTION);

	let submission = models::Submission {
		repo_name: repo_name.clone(),
		commit_sha,
		logstream_id,
		logstream_url,
		relationships: vec![],
		created_at: chrono::Utc::now(),
	};

	info!("Inserting submission for repository `{}` into database", repo_name);

	collection
		.insert_one(submission)
		.await
		.map(|_| {
			info!("Successfully inserted submission for repository `{}` into database", repo_name)
		})
		.map_err(SubmissionCreationError::from)
}
