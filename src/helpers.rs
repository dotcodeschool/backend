use actix_web::HttpResponse;

use crate::{
	errors::{DbError, RepoCreationError},
	models::{Course, Repository},
	types::{CreateRepoResponse, CreateSubmissionResponse, UpdateRepoResponse},
};

/// Constructs an HTTP response for a successful course data retrieval
pub(super) fn fetch_course_success_response(course: Course) -> HttpResponse {
	HttpResponse::Ok().json(course)
}

/// Constructs an HTTP response for a successful repository creation
pub(super) fn repository_creation_success_response(
	repo_name: String,
	template: &str,
) -> HttpResponse {
	HttpResponse::Ok().json(CreateRepoResponse { repo_name, repo_template: template.to_string() })
}

/// Handles errors during repository creation and returns the appropriate HTTP response
pub(super) fn handle_repo_creation_error(error: RepoCreationError) -> HttpResponse {
	match error {
		RepoCreationError::GitServerError(_) =>
			HttpResponse::InternalServerError().body("Failed to communicate with git server"),
		RepoCreationError::DatabaseError(_) =>
			HttpResponse::InternalServerError().body("Failed to save repository to database"),
		RepoCreationError::InvalidObjectId(_) =>
			HttpResponse::InternalServerError().body("Invalid object id"),
		RepoCreationError::InsertionError(_) =>
			HttpResponse::InternalServerError().body("Failed to insert repository into database"),
		RepoCreationError::NotFound(_) => HttpResponse::NotFound().body("404 Not Found"),
		RepoCreationError::InternalServerError(_) =>
			HttpResponse::InternalServerError().body("500 Internal Server Error"),
	}
}

/// Constructs an HTTP response for a successful submission creation
pub(super) fn submission_creation_success_response(
	submission_reponse: CreateSubmissionResponse,
) -> HttpResponse {
	HttpResponse::Ok().json(submission_reponse)
}

/// Handles errors during submission creation and returns the appropriate HTTP response
pub(super) fn handle_db_error(error: DbError) -> HttpResponse {
	match error {
		DbError::DatabaseError(_) =>
			HttpResponse::InternalServerError().body("Failed to save submission to database"),
		DbError::InternalServerError(_) =>
			HttpResponse::InternalServerError().body("500 Internal Server Error"),
		DbError::NotFound(_) => HttpResponse::NotFound().body("404 Not Found"),
	}
}

/// Constructs an HTTP response for successful retrieval of repository
pub(super) fn get_repository_success_response(repository: Repository) -> HttpResponse {
	HttpResponse::Ok().json(repository)
}

/// Constructs an HTTP response for a successful repository update
pub(super) fn repository_update_success_response(repository: Repository) -> HttpResponse {
	HttpResponse::Ok().json(UpdateRepoResponse {
		repo_name: repository.repo_name,
		repo_template: repository.repo_template,
		tester_url: repository.tester_url,
		test_ok: repository.test_ok,
		relationships: repository.relationships,
		expected_practice_frequency: repository.expected_practice_frequency,
		is_reminder_enabled: repository.is_reminder_enabled,
	})
}
