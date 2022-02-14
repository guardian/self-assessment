pub mod models;
pub mod helpers;
use crate::models::Args;
use clap::StructOpt;
use helpers::*;
use models::GuardianPullRequests;
use octocrab::Octocrab;
use std::error::Error;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, self};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Check if auth token exists in the user's system. If not, it must be passed
    // as an env variable when running the executable binary
    let auth_token = get_auth_token()?;
    let octocrab = Octocrab::builder().personal_token(auth_token).build()?;

    let user = octocrab.current().user().await?;
    let mut params = prepare_parameters();

    // Query the Github API with custom queries
    let authored_prs = search_pull_requests(
        &octocrab,
        GuardianPullRequests::AuthoredByMe,
        &mut params,
        &args,
    )
    .await;
    let reviewed_prs = search_pull_requests(
        &octocrab,
        GuardianPullRequests::ReviewedByMe,
        &mut params,
        &args,
    )
    .await;

    let formatted_prs = format_prs(&authored_prs);
    let formatted_reviews = format_prs(&reviewed_prs);

    // Generate HTML file
    let output_file_name = "self-assessment.html";
    let html_file = generate_html_file(user, &formatted_prs, &formatted_reviews, &args);

    // Automatically open the file if the operation succeeds
    match html_file {
        Ok(_) => {
            let mut open = Command::new("open");
            open.arg(output_file_name);
            open.exec();
        },
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        },
    }

    Ok(())
}
