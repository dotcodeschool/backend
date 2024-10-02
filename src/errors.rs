use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoCreationError {
	#[error("Git server request failed: {0}")]
	GitServerError(#[from] reqwest::Error),

	#[error("Database operation failed: {0}")]
	DatabaseError(#[from] mongodb::error::Error),

	#[error("Invalid object id: {0}")]
	InvalidObjectId(#[from] mongodb::bson::oid::Error),

	#[error("Insertion error: {0}")]
	InsertionError(String),

	#[error("404 Not Found: {0}")]
	NotFound(String),

	#[error("500 Internal Server Error: {0}")]
	InternalServerError(String),
}

#[derive(Error, Debug)]
pub enum DbError {
	#[error("Database operation failed: {0}")]
	DatabaseError(#[from] mongodb::error::Error),

	#[error("500 Internal Server Error: {0}")]
	InternalServerError(String),

	#[error("404 Not Found: {0}")]
	NotFound(#[from] actix_web::error::Error),
}
