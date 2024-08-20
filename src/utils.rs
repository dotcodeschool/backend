use crate::constants::{COLLECTION_NAME, DB_NAME, GIT_SERVER_URL};
use crate::errors::RepoCreationError;
use crate::models;
use crate::models::Repository;
use crate::types::CreateRepoRequest;
use crate::types::DocumentType;
use log::{info, warn};
use mongodb::Client;
use rand::prelude::*;
use std::collections::HashMap;

/// Generate a repository ID
/// This is used to identify the repository on the git server
pub(super) fn generate_repo_id() -> String {
    // Generate a random 16 character string
    let mut repo_id = hex::encode(rand::thread_rng().gen::<[u8; 16]>()).to_string();
    repo_id.truncate(16);
    repo_id
}

/// Create a repository on the git server and insert it into the database
pub(super) async fn do_create_repo(
    client: &Client,
    json: &CreateRepoRequest,
) -> Result<String, RepoCreationError> {
    let repo_name = generate_repo_id();
    let repo_template = json.repo_template.clone();
    let user_id = json.user_id.clone();

    info!(
        "Creating repository `{}` using template `{}`",
        repo_name, &repo_template
    );

    create_git_repo(&repo_name, &repo_template).await?;
    insert_repo_into_db(client, &repo_name, &repo_template, &user_id).await?;

    info!(
        "Successfully created repository `{}` on git server",
        repo_name
    );

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
    user_id: &str,
) -> Result<(), RepoCreationError> {
    let collection = client.database(DB_NAME).collection(COLLECTION_NAME);

    let repository = Repository {
        repo_name: repo_name.to_string(),
        repo_template: template.to_string(),
        relationships: vec![models::Relationship {
            id: user_id.to_string(),
            r#type: DocumentType::User,
        }],
    };

    collection
        .insert_one(repository)
        .await
        .map(|_| {
            info!(
                "Successfully inserted repository `{}` into database",
                repo_name
            )
        })
        .map_err(RepoCreationError::from)
}

/// Add a bearer token to the request if available
fn add_bearer_token_if_available(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    match std::env::var("BEARER_TOKEN_SECRET") {
        Ok(token) => {
            info!("Using bearer token to authenticate with git server");
            request.bearer_auth(token)
        }
        Err(_) => {
            warn!("No bearer token found, proceeding without authentication");
            request
        }
    }
}
