use std::collections::HashMap;

use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{types::DocumentType, ExpectedPracticeFrequency};

/// A repository document. This is used to store information about the owner of the repository, the
/// template used to create the repository, and the relationships between the repository and other
/// documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
	pub repo_name: String,
	pub repo_template: String,
	pub tester_url: String,
	pub relationships: HashMap<String, Relationship>,
	pub expected_practice_frequency: ExpectedPracticeFrequency,
	pub is_reminder_enabled: bool,
}

/// A user document. This is used to store information about the user, the repositories they own,
/// and the relationships between the user and other documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
	pub name: String,
	pub repositories: Vec<Relationship>,
	pub relationships: Vec<Relationship>,
}

/// A course document. This is used to store information about the course, the users enrolled in the
/// course, and the relationships between the course and other documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub slug: String,
	pub name: String,
	pub tester_url: String,
	#[serde(default)]
	pub relationships: Vec<Relationship>,
}

/// A relationship between documents. This is used to store the ID of the document and the type of
/// document in the relationship.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
	pub id: ObjectId,
	pub r#type: DocumentType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submission {
	pub repo_name: String,
	pub commit_sha: String,
	pub logstream_id: String,
	pub logstream_url: String,
	pub relationships: Vec<Relationship>,
	pub created_at: chrono::DateTime<Utc>,
}
