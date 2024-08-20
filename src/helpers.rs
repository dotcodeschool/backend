use crate::errors::RepoCreationError;
use actix_web::HttpResponse;

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
        RepoCreationError::GitServerError(_) => {
            HttpResponse::InternalServerError().body("Failed to communicate with git server")
        }
        RepoCreationError::DatabaseError(_) => {
            HttpResponse::InternalServerError().body("Failed to save repository to database")
        }
    }
}
