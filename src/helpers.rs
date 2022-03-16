use crate::cli::AuthFlag;
use crate::models::*;
use chrono::{DateTime, Datelike};
use colorsys::{Hsl, Rgb};
use handlebars::{to_json, Context, Handlebars, Helper, Output, RenderContext, RenderError};
use ini::Ini;
use octocrab::Octocrab;
use serde_json::Map;
use std::error::Error;
use std::fs::File;
use std::process;
use std::{borrow::Cow, collections::HashMap};
use url::Url;

pub fn exit_upon_setting_credentials(args: &crate::cli::Args) {
    if args.auth_token.is_some() {
        println!("[self-assessment] 🔑 GitHub personal access token set successfully.");
        process::exit(0);
    }

    if args.trello_key.is_some() {
        println!("[self-assessment] 🔑 Trello API key set successfully.");
        process::exit(0);
    }

    if args.trello_token.is_some() {
        println!("[self-assessment] 🔑 Trello server token set successfully.");
        process::exit(0);
    }
}

pub fn get_auth_token(args: &crate::cli::Args, flag: AuthFlag) -> Option<String> {
    // Attempt to load auth credentials from disk
    let credential_store_path = format!("{}/.selfassessment", shellexpand::tilde("~/"));
    let mut credential_store = match Ini::load_from_file(&credential_store_path) {
        Ok(ini) => ini,
        Err(_) => Ini::new(),
    };

    match flag {
        AuthFlag::GitHubAuthToken => {
            if args.auth_token.is_some() {
                let github_auth_token = args.auth_token.as_ref().unwrap();
                credential_store
                    .with_section(Some("GitHub"))
                    .set("GITHUB_TOKEN", github_auth_token);
                match credential_store.write_to_file(&credential_store_path) {
                    Ok(_) => Some(String::from(github_auth_token)),
                    Err(err) => panic!("GitHub auth token error: {}", err),
                }
            } else {
                credential_store
                    .with_section(Some("GitHub"))
                    .get("GITHUB_TOKEN")
                    .map(|t| t.to_string())
            }
        }
        AuthFlag::TrelloApiKey => {
            if args.trello_key.is_some() {
                let trello_key = args.trello_key.as_ref().unwrap();
                credential_store
                    .with_section(Some("Trello"))
                    .set("TRELLO_KEY", trello_key);
                match credential_store.write_to_file(&credential_store_path) {
                    Ok(_) => Some(String::from(trello_key)),
                    Err(err) => panic!("{}", err),
                }
            } else {
                credential_store
                    .with_section(Some("Trello"))
                    .get("TRELLO_KEY")
                    .map(|t| t.to_string())
            }
        }
        AuthFlag::TrelloServerToken => {
            if args.trello_token.is_some() {
                let trello_token = args.trello_token.as_ref().unwrap();
                credential_store
                    .with_section(Some("Trello"))
                    .set("TRELLO_TOKEN", trello_token);
                match credential_store.write_to_file(&credential_store_path) {
                    Ok(_) => Some(String::from(trello_token)),
                    Err(err) => panic!("{}", err),
                }
            } else {
                credential_store
                    .with_section(Some("Trello"))
                    .get("TRELLO_TOKEN")
                    .map(|t| t.to_string())
            }
        }
    }
}

pub fn prepare_parameters<'a>() -> HashMap<&'static str, Cow<'a, str>> {
    let mut params: HashMap<&'static str, Cow<str>> = HashMap::new();
    params.insert("per_page", Cow::from("100"));
    params.insert("page", Cow::from("1"));
    params.insert("order", Cow::from("desc"));
    params
}

// Reverse engineer Github label CSS magic
pub fn calc_label_colour(colour: &str) -> String {
    let rgb = Rgb::from_hex_str(colour).unwrap();
    let perceived_lightness =
        (rgb.red() * 0.2126 + rgb.green() * 0.7152 + rgb.blue() * 0.0722) / 255.0;
    let lightness_switch = f64::max(0.0, f64::min(1.0, (perceived_lightness - 0.453) * -1000.0));
    let hsl = Hsl::new(0.0, 0.0, lightness_switch * 100.0, Some(1.0));
    hsl.to_css_string()
}

pub async fn search_pull_requests<'a>(
    client: &Octocrab,
    pr_type: GuardianPullRequests,
    params: &mut HashMap<&'static str, Cow<'a, str>>,
    args: &crate::cli::Args,
) -> Vec<GithubSearchResponseItem> {
    let mut all_results: Vec<GithubSearchResponseItem> = vec![];
    let mut count = 1;

    match pr_type {
        GuardianPullRequests::AuthoredByMe => {
            params.insert(
                "q",
                Cow::from(format!(
                    "author:@me org:guardian created:{}..{}",
                    args.from, args.to
                )),
            );
        }
        GuardianPullRequests::ReviewedByMe => {
            params.insert(
                "q",
                Cow::from(format!(
                    "reviewed-by:@me -author:@me org:guardian created:{}..{}",
                    args.from, args.to
                )),
            );
        }
    }

    loop {
        println!(
            "[self-assessment] {} Collecting {}...",
            match pr_type {
                GuardianPullRequests::AuthoredByMe => "🔎",
                GuardianPullRequests::ReviewedByMe => "🔍",
            },
            pr_type
        );

        params.insert("page", Cow::from(count.to_string()));
        let result: Result<GithubSearchResponse, octocrab::Error> =
            client.get("search/issues", Some(&params)).await;
        match result {
            Ok(mut response) => {
                all_results.append(&mut response.items);
                count += 1;
                if all_results.len() == response.total_count as usize {
                    break;
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    }

    all_results
}

pub async fn search_trello_user(
    trello_client: &reqwest::Client,
    trello_key: String,
    trello_token: String,
) -> Result<TrelloUser, Box<dyn Error>> {
    let trello_user: TrelloUser = trello_client
        .get(format!(
            "https://api.trello.com/1/members/me?key={}&token={}&fields=avatarUrl,id,fullName",
            &trello_key, &trello_token
        ))
        .send()
        .await?
        .json()
        .await?;

    Ok(trello_user)
}

pub async fn search_trello(
    trello_client: &reqwest::Client,
    trello_key: String,
    trello_token: String,
    trello_user: &TrelloUser,
    args: &crate::cli::Args,
) -> Result<HashMap<String, Vec<TrelloCard>>, Box<dyn Error>> {
    println!("[self-assessment] 🃏 Collecting your Trello cards...");
    let response: Vec<TrelloBoard> = trello_client
        .get(format!(
            "https://api.trello.com/1/members/me/boards?key={}&token={}&fields=id,name",
            &trello_key, &trello_token
        ))
        .send()
        .await?
        .json()
        .await?;

    let board_ids = response
        .iter()
        .map(|x| (x.id.clone(), x.name.clone()))
        .collect::<HashMap<String, String>>();

    let mut trello_cards: HashMap<String, Vec<TrelloCard>> = HashMap::new();

    for (board_id, board_name) in board_ids {
        let all_cards_in_board: Vec<TrelloCard> = trello_client
        .get(format!(
            "https://api.trello.com/1/boards/{}/cards?key={}&token={}&fields=url,idMembers,name,desc,dateLastActivity,labels", 
            board_id,
            &trello_key,
            &trello_token
        ))
        .send()
        .await?
        .json()
        .await?;

        // Only collect trello cards you're assigned to
        let my_cards_only: Vec<TrelloCard> = all_cards_in_board
            .into_iter()
            .filter(|card| card.id_members.contains(&trello_user.id))
            .filter(|card| trello_cards_date_range(card, args))
            .collect();

        if !my_cards_only.is_empty() {
            trello_cards.insert(board_name, my_cards_only);
        }
    }

    Ok(trello_cards)
}

pub fn array_length_helper(
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

pub fn trello_cards_date_range(card: &TrelloCard, args: &crate::cli::Args) -> bool {
    let from = match args.from.as_str() {
        "*" => true,
        _ => {
            card.date_last_activity
                >= DateTime::parse_from_rfc3339(&format!("{}T00:00:00.00Z", args.from)).unwrap()
        }
    };

    let to = match args.to.as_str() {
        "*" => true,
        _ => {
            card.date_last_activity
                <= DateTime::parse_from_rfc3339(&format!("{}T00:00:00.00Z", args.to)).unwrap()
        }
    };

    from && to
}

// Returns a tuple containing the number of boards and the number of total cards across all boards
pub fn trello_board_and_cards_len(boards_with_cards: &[BoardAndCards]) -> (usize, usize) {
    let board_size = boards_with_cards.len();
    let total_cards = boards_with_cards
        .iter()
        .map(|x| x.cards.len())
        .sum::<usize>();
    (board_size, total_cards)
}

pub fn format_prs(results: &[GithubSearchResponseItem]) -> Vec<TemplatePr> {
    static OPEN_PR: &str = "<svg style=\"color: #1a7f37; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
    aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M7.177 3.073L9.573.677A.25.25 0 0110 .854v4.792a.25.25 
    0 01-.427.177L7.177 3.427a.25.25 0 010-.354zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 
    113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zM11 2.5h-1V4h1a1 1 0 011 1v5.628a2.251 2.251 
    0 101.5 0V5A2.5 2.5 0 0011 2.5zm1 10.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0zM3.75 12a.75.75 0 100 1.5.75.75 0 
    000-1.5z\"></path></svg>";

    static CLOSED_PR: &str = "<svg style=\"color: #8250df; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
    aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M5 3.254V3.25v.005a.75.75 0 110-.005v.004zm.45 1.9a2.25 2.25 
    0 10-1.95.218v5.256a2.25 2.25 0 101.5 0V7.123A5.735 5.735 0 009.25 9h1.378a2.251 2.251 0 100-1.5H9.25a4.25 4.25 0
     01-3.8-2.346zM12.75 9a.75.75 0 100-1.5.75.75 0 000 1.5zm-8.5 4.5a.75.75 0 100-1.5.75.75 0 000 1.5z\"></path></svg>";

    let base = Url::parse("https://api.github.com/repos/guardian/").unwrap();

    results
    .iter()
    .map(|r| {
        let url = Url::parse(&r.repository_url.to_string()).unwrap();
        let repo_name = base.make_relative(&url).unwrap();
        let status = match r.state.as_str() {
            "open" => OPEN_PR.to_string(),
            "closed" => CLOSED_PR.to_string(),
            _ => r.state.to_string(),
        };

        TemplatePr {
            status,
            created_at: r.created_at.split('T').collect::<Vec<&str>>()[0].to_string(),
            title: r.title.to_string(),
            html_url: r.html_url.to_string(),
            repo_name,
            comments: r.comments,
            comments_present: (r.comments > 0, r.comments == 1),
            body: markdown::to_html(match &r.body {
                Some(body) => body,
                None => "*No description provided.*",
            }),
            labels: r.labels.iter()
                .map(|l| format!("<span class=\"label\" style=\"color:{}; background-color: #{};\">{}</span>",
                calc_label_colour(&String::from(&l.color)),&l.color,&l.name))
                .collect::<Vec<String>>()
                .join(" "),
            author: r.user.login.to_string(),
            profile_pic: r.user.avatar_url.to_string(),
        }
    })
    .collect()
}

fn template_card_from_unformatted_card(card: &TrelloCard) -> TemplateTrelloCard {
    TemplateTrelloCard {
        name: card.name.to_string(),
        url: card.url.to_string(),
        labels: card
            .labels
            .iter()
            .map(|l| {
                format!(
                    "<span class=\"card-label\" style=\"background-color:{}\"><span>{}</span></span>",
                    match &l.color {
                        Some(c) => match c.as_str() {
                            "green" => "#61bd4f",
                            "yellow" => "#f2d600",
                            "orange" => "#ff9f1a",
                            "red" => "#eb5a46",
                            "purple" => "#c377e0",
                            "blue" => "#0079bf",
                            "sky" => "#00c2e0",
                            "lime" => "#51e898",
                            "pink" => "#ff78cb",
                            "black" => "#344563",
                            _ => "#344563",
                        },
                        None => "",
                    }, &l.name
                )
            })
            .collect::<Vec<String>>()
            .join(" "),
        }
}

pub fn format_trello_cards(cards: &HashMap<String, Vec<TrelloCard>>) -> Vec<BoardAndCards> {
    let mut formatted_cards: HashMap<String, Vec<TemplateTrelloCard>> = HashMap::new();
    for (key, value) in cards.iter() {
        formatted_cards.insert(
            key.to_string(),
            value
                .iter()
                .map(template_card_from_unformatted_card)
                .collect::<Vec<TemplateTrelloCard>>(),
        );
    }
    formatted_cards
        .into_iter()
        .map(|(board, cards)| BoardAndCards { board, cards })
        .collect()
}

pub fn generate_html_file(
    user: octocrab::models::User,
    prs: &[TemplatePr],
    reviews: &[TemplatePr],
    trello_user: &Option<TrelloUser>,
    trello_board_with_cards: &Option<Vec<BoardAndCards>>,
    args: &crate::cli::Args,
) -> Result<(), Box<dyn Error>> {
    let mut reg = Handlebars::new();
    reg.register_helper("array_length", Box::new(array_length_helper));
    reg.register_template_string("template", TEMPLATE).unwrap();

    // Write HTML template into binary
    static TEMPLATE: &str = include_str!("./template/template.hbs");

    let from = if args.from == "*" {
        "From the day you joined the Guardian".to_string()
    } else {
        format!("From {}", args.from)
    };
    let to = if args.to == "*" {
        "until today".to_string()
    } else {
        format!("to {}", args.to)
    };

    let mut data = Map::new();

    // GitHub Template
    data.insert("github_user".to_string(), to_json(user.login));
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
        data.insert("start_date".to_string(), to_json(from));
        data.insert("end_date".to_string(), to_json(to));
        data.insert("display_trello".to_string(), to_json(true));
        board_len = b_len;
    }

    let now = chrono::Utc::now();
    let output_file_name = format!(
        "[{}-{:02}-{:02}] self-assessment.html",
        now.year_ce().1,
        now.month(),
        now.day()
    );
    
    let mut output_file = File::create(output_file_name)?;
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

    Ok(())
}
