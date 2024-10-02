use std::collections::HashMap;

use log::{error, info, warn};
use mongodb::{
	bson::{self, doc, oid::ObjectId},
	Client,
};
use rand::prelude::*;

use crate::{
	constants::{DB_NAME, GIT_SERVER_URL, REPO_COLLECTION, SUBMISSION_COLLECTION, USER_COLLECTION},
	errors::{DbError, RepoCreationError},
	models::{self, Course, Repository},
	types::{CreateRepoRequest, CreateSubmissionRequest, CreateSubmissionResponse, DocumentType},
	ExpectedPracticeFrequency,
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

/// Fetch course data from database
pub(super) async fn fetch_course(client: &Client, id: &str) -> Result<Course, DbError> {
	let collection = client.database(DB_NAME).collection("courses");
	let id = ObjectId::parse_str(id).map_err(|e| {
		error!("Invalid ObjectId: {}", id);
		DbError::InternalServerError(e.to_string())
	})?;

	let filter = doc! { "_id": id };
	let course = collection.find_one(filter).await?;

	log::debug!("{:#?}", course);

	let result = match course {
		Some(course) => match bson::from_document::<Course>(course) {
			Ok(course) => {
				info!("Fetched course: {:?}", course);
				Ok(course)
			},
			Err(e) => {
				error!("Failed to deserialize course: {}", e);
				Err(DbError::InternalServerError(format!(
					"Failed to deserialize course with id {}",
					id
				)))
			},
		},
		None => Err(DbError::InternalServerError(format!("Course with id {} not found", id))),
	};

	log::debug!("{:#?}", result);
	result
}

/// Create a repository on the git server and insert it into the database
pub(super) async fn do_create_repo(
	client: &Client,
	json: &CreateRepoRequest,
) -> Result<String, RepoCreationError> {
	let repo_name = generate_repo_id();
	let repo_template = json.repo_template.clone();
	let user_id = json.user_id.clone();
	let user_id = ObjectId::parse_str(&user_id).map_err(|e| {
		error!("Invalid ObjectId: {}", user_id);
		RepoCreationError::InvalidObjectId(e)
	})?;
	let expected_practice_frequency = json.expected_practice_frequency.clone();
	let is_reminder_enabled = json.is_reminder_enabled;

	info!("Creating repository `{}` using template `{}` with expected practice frequency `{}` and reminders `{}`", repo_name, &repo_template, &expected_practice_frequency, &is_reminder_enabled);

	create_git_repo(&repo_name, &repo_template).await?;
	let repo_id = insert_repo_into_db(
		client,
		&repo_name,
		&repo_template,
		&user_id,
		expected_practice_frequency,
		is_reminder_enabled,
	)
	.await?;
	update_user_repo_list(client, &user_id, repo_id).await?;

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
	user_id: &ObjectId,
	expected_practice_frequency: ExpectedPracticeFrequency,
	is_reminder_enabled: bool,
) -> Result<mongodb::bson::oid::ObjectId, RepoCreationError> {
	let collection = client.database(DB_NAME).collection(REPO_COLLECTION);
	let course_id = get_course_id_by_slug(client, template).await?;

	let mut relationships = HashMap::new();
	relationships.insert(
		"user".to_string(),
		models::Relationship { id: *user_id, r#type: DocumentType::User },
	);
	relationships.insert(
		"course".to_string(),
		models::Relationship { id: course_id, r#type: DocumentType::Course },
	);

	let repository = Repository {
		repo_name: repo_name.to_string(),
		repo_template: template.to_string(),
		// TODO: Use the correct URL based on the template
		tester_url: "https://github.com/dotcodeschool/rust-state-machine-tester".to_string(),
		relationships,
		expected_practice_frequency,
		is_reminder_enabled,
	};

	let result = collection.insert_one(repository).await?;

	result.inserted_id.as_object_id().ok_or_else(|| {
		error!("Failed to get ObjectId after inserting repository `{}` into database", repo_name);
		RepoCreationError::InsertionError("Failed to get ObjectId after insertion".into())
	})
}

/// Get the course ID using the course slug
pub(super) async fn get_course_id_by_slug(
	client: &Client,
	slug: &str,
) -> Result<ObjectId, RepoCreationError> {
	let collection = client.database(DB_NAME).collection("courses");

	let filter = doc! { "slug": slug };
	let course = collection.find_one(filter).await?;

	match course {
		Some(course) => {
			let course: models::Course = match mongodb::bson::de::from_document(course) {
				Ok(course) => course,
				Err(e) => {
					error!("Failed to deserialize course: {}", e);
					return Err(RepoCreationError::InternalServerError(e.to_string()));
				},
			};
			Ok(course.id)
		},
		None => {
			warn!("Course with slug `{}` not found", slug);
			Err(RepoCreationError::NotFound(slug.to_string()))
		},
	}
}

/// Update the user's repository list in the database
pub(super) async fn update_user_repo_list(
	client: &Client,
	user_id: &ObjectId,
	repo_id: ObjectId,
) -> Result<(), RepoCreationError> {
	let collection: mongodb::Collection<models::User> =
		client.database(DB_NAME).collection(USER_COLLECTION);

	let filter = doc! { "_id": user_id };
	let update = doc! { "$addToSet": { "relationships.repositories.data": { "id": repo_id, "type": "repositories" }} };

	collection
		.update_one(filter, update)
		.await
		.map(|_| info!("Successfully updated user `{}` with repository `{}`", user_id, repo_id))
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
	ws_url: &str,
	json: &CreateSubmissionRequest,
) -> Result<CreateSubmissionResponse, DbError> {
	let repo_name = &json.repo_name;
	let commit_sha = &json.commit_sha;
	let ws_url = ws_url.to_string();

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

	Ok(CreateSubmissionResponse { logstream_id, logstream_url, ws_url, tester_url })
}

/// Fetch a repository from the database. Fail if the repository does not exist.
pub(super) async fn get_repo_from_db(
	client: &Client,
	repo_name: &str,
) -> Result<Repository, DbError> {
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
			Err(DbError::NotFound(actix_web::error::ErrorNotFound(format!(
				"Repository `{}` not found",
				repo_name
			))))
		},
		Err(e) => {
			error!("Error fetching repository `{}` from database: {:?}", repo_name, e);
			Err(DbError::DatabaseError(e))
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
) -> Result<(), DbError> {
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
		.map_err(DbError::from)
}
