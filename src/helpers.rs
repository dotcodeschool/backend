use actix_web::HttpResponse;

use crate::{
	errors::{RepoCreationError, SubmissionCreationError},
	types::CreateSubmissionResponse,
};

/// Constructs an HTTP response for a successful repository creation
pub(super) fn repository_creation_success_response(
	repo_name: String,
	template: &str,
) -> HttpResponse {
	HttpResponse::Ok().json({
		format!(
			"Successfully created repository `{}` using template `{}` on the git server.",
			repo_name, template
		)
	})
}

/// Handles errors during repository creation and returns the appropriate HTTP response
pub(super) fn handle_repo_creation_error(error: RepoCreationError) -> HttpResponse {
	match error {
		RepoCreationError::GitServerError(_) =>
			HttpResponse::InternalServerError().body("Failed to communicate with git server"),
		RepoCreationError::DatabaseError(_) =>
			HttpResponse::InternalServerError().body("Failed to save repository to database"),
	}
}

/// Constructs an HTTP response for a successful submission creation
pub(super) fn submission_creation_success_response(
	submission_reponse: CreateSubmissionResponse,
) -> HttpResponse {
	HttpResponse::Ok().json(submission_reponse)
}

/// Handles errors during submission creation and returns the appropriate HTTP response
pub(super) fn handle_submission_creation_error(error: SubmissionCreationError) -> HttpResponse {
	match error {
		SubmissionCreationError::SubmissionCreationError(_) =>
			HttpResponse::InternalServerError().body("Failed to create submission"),
		SubmissionCreationError::DatabaseError(_) =>
			HttpResponse::InternalServerError().body("Failed to save submission to database"),
		SubmissionCreationError::NotFound(_) =>
            HttpResponse::NotFound().body("404 Not Found"),
	}
}
