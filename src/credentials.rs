use ini::Ini;

use crate::cli::AuthType;

/// Set GitHub and Trello credentials.
pub fn set_credentials(
    store: &mut Ini,
    store_path: String,
    section: String,
    key: String,
    value: String,
) -> anyhow::Result<()> {
    let key_msg = match key.as_str() {
        "GITHUB_TOKEN" => "GitHub personal access token",
        "TRELLO_KEY" => "Trello API key token",
        "TRELLO_TOKEN" => "Trello server token",
        _ => "",
    };
    store.with_section(Some(section)).set(key, value);
    if store.write_to_file(store_path).is_ok() {
        println!("[self-assessment] 🔑 {key_msg} set successfully.");
    }
    Ok(())
}

/// Load credentials from disk.
/// Credentials live in `~/.selfassessment`
pub fn get_auth_token(flag: AuthType) -> Option<String> {
    let credential_store_path = format!("{}/.selfassessment", shellexpand::tilde("~/"));
    let mut credential_store = match Ini::load_from_file(&credential_store_path) {
        Ok(ini) => ini,
        Err(_) => Ini::new(),
    };

    match flag {
        AuthType::GitHubAuthToken => credential_store
            .with_section(Some("GitHub"))
            .get("GITHUB_TOKEN")
            .map(|t| t.to_string()),
        AuthType::TrelloApiKey => credential_store
            .with_section(Some("Trello"))
            .get("TRELLO_KEY")
            .map(|t| t.to_string()),
        AuthType::TrelloServerToken => credential_store
            .with_section(Some("Trello"))
            .get("TRELLO_TOKEN")
            .map(|t| t.to_string()),
    }
}
