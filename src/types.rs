use serde::{Deserialize, Serialize};

/// The type of document. This is used to identify the type of document in the relationships between
/// documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
	Repository,
	User,
	Course,
}

#[derive(serde::Deserialize)]
pub struct CreateRepoRequest {
	pub repo_template: String,
	pub(super) user_id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateSubmissionRequest {
	pub repo_name: String,
	pub commit_sha: String,
}

#[derive(serde::Serialize)]
pub struct CreateSubmissionResponse {
	pub logstream_url: String,
	pub tester_repo_url: String,
}
