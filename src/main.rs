pub mod cli;
pub mod helpers;
pub mod models;
use clap::StructOpt;
use cli::Args;
use helpers::*;
// use octocrab::Octocrab;
// use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::prelude::CommandExt;
use std::process::{self, Command};

use crate::cli::AuthFlag;
use crate::models::{TrelloBoard, TrelloCard, TrelloUser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let trello_key = get_auth_token(&args, AuthFlag::TrelloApiKey);
    let trello_token = get_auth_token(&args, AuthFlag::TrelloServerToken);
    let github_auth_token = get_auth_token(&args, AuthFlag::GitHubAuthToken);
    //let octocrab = Octocrab::builder().personal_token(auth_token).build()?;

    if !(trello_key.is_some() && trello_token.is_some()) {
        return Ok(());
    }

    let trello_client = reqwest::ClientBuilder::new().build()?;
    println!("[self-assessment] 🗂️ Attempting to collect your Trello cards...");
    let response: Vec<TrelloBoard> = trello_client
        .get(format!(
            "https://api.trello.com/1/members/me/boards?key={}&token={}&fields=id,name",
            &trello_key.as_ref().unwrap(),
            &trello_token.as_ref().unwrap()
        ))
        .send()
        .await?
        .json()
        .await?;
    let board_ids = response
        .iter()
        .map(|x| (x.id.clone(), x.name.clone()))
        .collect::<HashMap<String, String>>();

    let trello_user: TrelloUser = trello_client
        .get(format!(
            "https://api.trello.com/1/members/me?key={}&token={}&fields=avatarUrl,id,fullName",
            &trello_key.as_ref().unwrap(),
            &trello_token.as_ref().unwrap()
        ))
        .send()
        .await?
        .json()
        .await?;

    let mut trello_cards: HashMap<String, Vec<TrelloCard>> = HashMap::new();

    for (board_id, board_name) in board_ids {
        let all_cards_in_board: Vec<TrelloCard> = trello_client
        .get(format!(
            "https://api.trello.com/1/boards/{}/cards?key={}&token={}&fields=url,idMembers,name,desc,dateLastActivity,labels", 
            board_id,
            &trello_key.as_ref().unwrap(),
            &trello_token.as_ref().unwrap()
        ))
        .send()
        .await?
        .json()
        .await?;

        // Only collect trello cards I'm assigned to
        let my_cards_only: Vec<TrelloCard> = all_cards_in_board
            .into_iter()
            .filter(|card| card.id_members.contains(&trello_user.id))
            .filter(|card| trello_cards_date_range(card, &args))
            .collect();

        if !my_cards_only.is_empty() {
            trello_cards.insert(board_name, my_cards_only);
        }
    }

    // let user = octocrab.current().user().await?;
    // let mut params = prepare_parameters();

    // // Query the Github API with custom queries
    // let authored_prs = search_pull_requests(
    //     &octocrab,
    //     GuardianPullRequests::AuthoredByMe,
    //     &mut params,
    //     &args,
    // )
    // .await;
    // let reviewed_prs = search_pull_requests(
    //     &octocrab,
    //     GuardianPullRequests::ReviewedByMe,
    //     &mut params,
    //     &args,
    // )
    // .await;

    // let formatted_prs = format_prs(&authored_prs);
    // let formatted_reviews = format_prs(&reviewed_prs);
    let formatted_trello_cards = format_trello_cards(&trello_cards);
    //formatted_trello_cards.reverse();

    // Generate HTML file
    // let output_file_name = "self-assessment.html";
    // let html_file = generate_html_file(user, &formatted_prs, &formatted_reviews, &args);

    let output_file_name = "self-assessment.html";
    let html_file = generate_html_file(trello_user, &formatted_trello_cards, &args);
    // Automatically open the file if the operation succeeds
    match html_file {
        Ok(_) => {
            let mut open = Command::new("open");
            open.arg(output_file_name);
            open.exec();
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }

    Ok(())
}
