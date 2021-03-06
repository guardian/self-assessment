pub mod cli;
pub mod helpers;
pub mod models;
use clap::StructOpt;
use cli::Args;
use helpers::*;
use octocrab::Octocrab;
use std::error::Error;
use std::os::unix::prelude::CommandExt;
use std::process::{self, Command};

use crate::cli::AuthFlag;
use crate::models::GuardianPullRequests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let github_auth_token = get_auth_token(&args, AuthFlag::GitHubAuthToken);
    let trello_key = get_auth_token(&args, AuthFlag::TrelloApiKey);
    let trello_token = get_auth_token(&args, AuthFlag::TrelloServerToken);

    // Configuring auth token triggers the end of the execution
    exit_upon_setting_credentials(&args);

    // The GitHub auth token is the minimum config parameter needed to run the CLI,
    // so exit if it's not found
    if github_auth_token.is_none() {
        eprintln!("[self-assessment] ❌ Unable to fetch the GitHub authentication token.");
        eprintln!(
            "[self-assessment] ❌ Please try and run the tool with the --auth-token flag again."
        );
        process::exit(1);
    }

    // GitHub HTTP client
    let octocrab = Octocrab::builder()
        .personal_token(github_auth_token.unwrap())
        .build()?;

    let github_user = octocrab.current().user().await?;
    let mut github_params = prepare_parameters();

    // Query the Github API with custom queries
    let authored_prs = search_pull_requests(
        &octocrab,
        GuardianPullRequests::AuthoredByMe,
        &mut github_params,
        &args,
    )
    .await;

    let reviewed_prs = search_pull_requests(
        &octocrab,
        GuardianPullRequests::ReviewedByMe,
        &mut github_params,
        &args,
    )
    .await;

    let formatted_prs = format_prs(&authored_prs);
    let formatted_reviews = format_prs(&reviewed_prs);

    // Trello integration
    let mut trello_user = None;
    let mut formatted_trello_cards = None;

    if trello_key.is_some() && trello_token.is_some() && !args.skip_trello {
        let trello_client = reqwest::ClientBuilder::new().build()?;
        let maybe_user = search_trello_user(
            &trello_client,
            trello_key.as_ref().unwrap().to_string(),
            trello_token.as_ref().unwrap().to_string(),
        )
        .await;

        match maybe_user {
            Ok(user) => {
                let trello_cards = search_trello(
                    &trello_client,
                    trello_key.as_ref().unwrap().to_string(),
                    trello_token.as_ref().unwrap().to_string(),
                    &user,
                    &args,
                )
                .await?;

                trello_user = Option::from(user);
                formatted_trello_cards = Option::from(format_trello_cards(&trello_cards));
            }
            Err(err) => {
                eprintln!("[self-assessment] 🚫 Trello error: \"{}\"", err);
                eprintln!("[self-assessment] 🚫 Make sure your Trello API key is correct and your server token hasn't expired. If the error persists, use the --skip-trello flag.");
            }
        }
    } else {
        println!("[self-assessment] ⏩ Skipping Trello report.");
    }

    // Generate HTML file
    let html_file = generate_html_file(
        github_user,
        &formatted_prs,
        &formatted_reviews,
        &trello_user,
        &formatted_trello_cards,
        &args,
    );

    // Automatically open the file if the operation succeeds
    match html_file {
        Ok(file_name) => {
            let mut open = Command::new("open");
            open.arg(file_name);
            open.exec();
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }

    Ok(())
}
