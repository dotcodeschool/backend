use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoCreationError {
	#[error("Git server request failed: {0}")]
	GitServerError(#[from] reqwest::Error),

	#[error("Database operation failed: {0}")]
	DatabaseError(#[from] mongodb::error::Error),
}
