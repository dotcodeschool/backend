use rand::prelude::*;

/// Generate a repository ID
/// This is used to identify the repository on the git server
pub(super) fn generate_repo_id() -> String {
    // Generate a random 16 character string
    let mut repo_id = hex::encode(rand::thread_rng().gen::<[u8; 16]>()).to_string();
    repo_id.truncate(16);
    repo_id
}
