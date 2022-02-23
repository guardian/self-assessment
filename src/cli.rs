use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug, Serialize)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Match PRs and Trello cards that were created after this date
    #[clap(short, long, default_value = "*")]
    pub from: String,

    /// Match PRs and Trello cards that were created up until this date
    #[clap(short, long, default_value = "*")]
    pub to: String,

    /// Github authentication token.
    /// This is needed for the CLI tool to access
    /// the Guardian's private repositories to which the user has access.
    /// You can get a personal access token at https://github.com/settings/tokens/new
    #[clap(short, long)]
    #[serde(rename = "auth-token")]
    pub auth_token: Option<String>,

    /// Trello API key.
    /// You can get an API key at https://trello.com/app-key
    /// Note: you need to be logged into Trello to be able to see the page.
    /// Both the API key and the server token need to be set for the generated report
    /// to include Trello cards.
    #[clap(long)]
    #[serde(rename = "trello-key")]
    pub trello_key: Option<String>,

    /// Trello server token.
    /// You can get a server token by following the link at https://trello.com/app-key
    /// Both the API key and the server token need to be set for the generated report
    /// to include Trello cards.
    #[clap(long)]
    #[serde(rename = "trello-token")]
    pub trello_token: Option<String>,
}

#[derive(Debug)]
pub enum AuthFlag {
    GitHubAuthToken,
    TrelloApiKey,
    TrelloServerToken,
}
