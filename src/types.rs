use crate::models::Relationship;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::Display;

/// The type of document. This is used to identify the type of document in the relationships between
/// documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
	Repository,
	User,
	Course,
}

/// Expected activity frequency for a repository. This is used to determine how often the user wants
/// to practice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedPracticeFrequency {
	EveryDay,
	OnceAWeek,
	OnceAMonth,
}

#[derive(serde::Deserialize)]
pub struct CreateRepoRequest {
	pub repo_template: String,
	pub(super) user_id: String,
	pub expected_practice_frequency: ExpectedPracticeFrequency,
	pub is_reminder_enabled: bool,
}

#[derive(serde::Serialize)]
pub struct CreateRepoResponse {
	pub repo_name: String,
	pub repo_template: String,
}

#[derive(serde::Deserialize)]
pub struct CreateSubmissionRequest {
	pub repo_name: String,
	pub commit_sha: String,
}

#[derive(serde::Serialize)]
pub struct CreateSubmissionResponse {
	pub logstream_url: String,
	pub logstream_id: String,
	pub ws_url: String,
	pub tester_url: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateRepoRequest {
	pub expected_practice_frequency: Option<ExpectedPracticeFrequency>,
	pub is_reminder_enabled: Option<bool>,
	pub test_ok: Option<bool>,
	pub relationships: Option<HashMap<String, Relationship>>,
}

#[derive(serde::Serialize)]
pub struct UpdateRepoResponse {
	pub repo_name: String,
	pub repo_template: String,
	pub tester_url: String,
	pub test_ok: Option<bool>,
	pub relationships: HashMap<String, Relationship>,
	pub expected_practice_frequency: ExpectedPracticeFrequency,
	pub is_reminder_enabled: bool,
}
