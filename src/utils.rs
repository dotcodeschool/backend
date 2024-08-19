use log::{info, warn};
use rand::prelude::*;
use std::collections::HashMap;

/// The URL of the git server
const GIT_SERVER_URL: &str = "https://git.dotcodeschool.com";

/// Generate a repository ID
/// This is used to identify the repository on the git server
pub(super) fn generate_repo_id() -> String {
    // Generate a random 16 character string
    let mut repo_id = hex::encode(rand::thread_rng().gen::<[u8; 16]>()).to_string();
    repo_id.truncate(16);
    repo_id
}

/// Send a post request to the git server to create a repository
pub(super) async fn create_git_repo(repo_name: &str, template: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v0/create_repository", GIT_SERVER_URL);

    let json: HashMap<&str, &str> =
        HashMap::from_iter(vec![("repo_name", repo_name), ("template_repo", template)]);

    let mut request = client.post(&url);

    match std::env::var("BEARER_TOKEN_SECRET") {
        Ok(token) => {
            request = request.bearer_auth(token);
            info!("Using bearer token to authenticate with git server");
        }
        Err(_) => {
            warn!("No bearer token found, skipping authentication with git server");
        }
    };

    let response = request.json(&json).send().await?;

    match response.error_for_status() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
