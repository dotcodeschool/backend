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
