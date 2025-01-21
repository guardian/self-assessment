use std::{
    fs::File,
    os::unix::process::CommandExt,
    process::{self, Command},
};

use chrono::Datelike;
use handlebars::{to_json, Context, Handlebars, Helper, Output, RenderContext, RenderError};
use octocrab::Octocrab;
use serde_json::Map;

use crate::cli::AuthType;
use crate::credentials::get_auth_token;
use crate::github::{format_prs, prepare_parameters, search_pull_requests};
use crate::models::{BoardAndCards, GuardianPullRequests, TemplatePr, TrelloUser};
use crate::trello::{
    format_trello_cards, search_trello, search_trello_user, trello_board_and_cards_len,
};

fn array_length_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let length = h
        .param(0)
        .as_ref()
        .and_then(|v| v.value().as_array())
        .map(|arr| arr.len())
        .ok_or_else(|| {
            RenderError::new("Param 0 with 'array' type is required for array_length helper")
        })?;

    out.write(length.to_string().as_ref())?;

    Ok(())
}

pub fn generate_html_file(
    user: octocrab::models::User,
    prs: &[TemplatePr],
    reviews: &[TemplatePr],
    trello_user: &Option<TrelloUser>,
    trello_board_with_cards: &Option<Vec<BoardAndCards>>,
    from: &str,
    to: &str,
) -> anyhow::Result<String> {
    let mut reg = Handlebars::new();
    reg.register_helper("array_length", Box::new(array_length_helper));
    reg.register_template_string("template", TEMPLATE).unwrap();

    // Write HTML template into binary
    static TEMPLATE: &str = include_str!("./template/template.hbs");

    let from = if from == "*" {
        "From the day you joined the Guardian".to_string()
    } else {
        format!("From {}", from)
    };
    let to = if to == "*" {
        "until today".to_string()
    } else {
        format!("to {}", to)
    };

    let mut data = Map::new();

    // GitHub Template
    data.insert("github_user".to_string(), to_json(user.login));
    data.insert("start_date".to_string(), to_json(from));
    data.insert("end_date".to_string(), to_json(to));
    data.insert("prs".to_string(), to_json(prs));
    data.insert("reviews".to_string(), to_json(reviews));
    data.insert("prs_len".to_string(), to_json(prs.len()));
    data.insert("reviews_len".to_string(), to_json(reviews.len()));
    data.insert(
        "trello_boards".to_string(),
        to_json(trello_board_with_cards),
    );

    // Trello Template
    let mut board_len: usize = 0;
    let mut c_len: usize = 0;

    if let (Some(u), Some(b)) = (trello_user, trello_board_with_cards) {
        let (b_len, cards_len) = trello_board_and_cards_len(b);
        c_len = cards_len;
        data.insert("cards_len".to_string(), to_json(cards_len));
        data.insert("user".to_string(), to_json(u));
        data.insert("display_trello".to_string(), to_json(true));
        board_len = b_len;
    }

    let now = chrono::Utc::now();
    let output_file_name = format!(
        "{}-{:02}-{:02}-self-assessment.html",
        now.year_ce().1,
        now.month(),
        now.day()
    );

    let mut output_file = File::create(&output_file_name)?;
    reg.render_to_write("template", &data, &mut output_file)?;

    println!(
        "[self-assessment] ✨ Generated a report containing {} PRs ({} authored, {} reviewed)",
        prs.len() + reviews.len(),
        prs.len(),
        reviews.len()
    );
    if trello_board_with_cards.is_some() {
        println!(
            "[self-assessment] ✨ ...including {} cards in {} Trello boards",
            c_len, board_len
        )
    }

    Ok(output_file_name)
}

pub async fn generate_report(from: String, to: String, skip_trello: bool) -> anyhow::Result<()> {
    let github_auth_token = get_auth_token(AuthType::GitHubAuthToken);
    let trello_key = get_auth_token(AuthType::TrelloApiKey);
    let trello_token = get_auth_token(AuthType::TrelloServerToken);

    // The GitHub auth token is the minimum config parameter needed to run the CLI,
    // so exit if it's not found
    if github_auth_token.is_none() {
        eprintln!("[self-assessment] ❌ Unable to fetch the GitHub authentication token.");
        eprintln!("[self-assessment] ❌ Run `self-assessment auth <TOKEN>`");
        std::process::exit(1);
    }

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
        &from,
        &to,
    )
    .await?;

    let reviewed_prs = search_pull_requests(
        &octocrab,
        GuardianPullRequests::ReviewedByMe,
        &mut github_params,
        &from,
        &to,
    )
    .await?;

    let formatted_prs = format_prs(&authored_prs);
    let formatted_reviews = format_prs(&reviewed_prs);

    // Trello integration
    let mut trello_user = None;
    let mut formatted_trello_cards = None;

    if trello_key.is_some() && trello_token.is_some() && !skip_trello {
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
                    &from,
                    &to,
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
        &from,
        &to,
    );

    // Automatically open the file if the operation succeeds
    match html_file {
        Ok(file_name) => {
            let mut open = Command::new("open");
            open.arg(file_name);
            // Exec the opening of the file.
            // If all goes well, this will never return.
            // If it does return, it will always retun an error.
            let err = open.exec();
            eprintln!("Error opening file: {}", err);
            process::exit(1);
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
