extern crate handlebars;
use crate::models::{self, GithubSearchResponse, GithubSearchResponseItem, TemplatePr};
use colorsys::{Hsl, Rgb};
use handlebars::{to_json, Handlebars};
use ini::Ini;
use models::GuardianPullRequests;
use octocrab::Octocrab;
use serde_json::Map;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::{borrow::Cow, collections::HashMap};
use url::Url;

pub fn get_auth_token(args: &crate::models::Args) -> Result<String, Box<dyn Error>> {
    let env_var_path = format!("{}/.selfassessment", shellexpand::tilde("~/"));

    if !args.auth_token.is_empty() {
        let mut env_var_file = File::create(env_var_path)?;
        match env_var_file.write_all(format!("GITHUB_TOKEN={}", args.auth_token).as_bytes()) {
            Ok(_) => println!("[self-assessment] 🔑 Personal access token set successfully."),
            Err(_) => eprintln!("[self-assessment] ❌ Unable to set authentication credentials."),
        };
        process::exit(0);
    }

    if Path::new(&env_var_path).exists() {
        let mut conf = Ini::load_from_file(&env_var_path).unwrap();
        match conf.with_general_section().get("GITHUB_TOKEN") {
            Some(t) => std::env::set_var("GITHUB_TOKEN", t),
            None => {
                eprintln!("[self-assessment] ❌ Unable to fetch authentication credentials.");
                eprintln!("[self-assessment] ❌ Please try and run the tool with the --auth-token flag again.");
                process::exit(1);
            }
        }
    }

    match std::env::var("GITHUB_TOKEN") {
        Ok(t) => Ok(t),
        Err(_) => {
            eprintln!("[self-assessment] ❌ Unable to fetch authentication credentials.");
            eprintln!(
                "[self-assessment] ❌ Please authenticate the tool with the --auth-token flag."
            );
            process::exit(1);
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

// Reverse engineered the Github label CSS magic... I think this works???
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
    args: &crate::models::Args,
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
        params.insert("page", Cow::from(count.to_string()));
        let result: Result<GithubSearchResponse, octocrab::Error> =
            client.get("search/issues", Some(&params)).await;
        match result {
            Ok(mut response) => {
                if !response.items.is_empty() {
                    all_results.append(&mut response.items);
                    count += 1;
                } else {
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

pub fn format_prs(results: &[GithubSearchResponseItem]) -> Vec<TemplatePr> {
    results
    .iter()
    .map(|r| {
        let base = Url::parse("https://api.github.com/repos/guardian/").unwrap();
        let url = Url::parse(&r.repository_url.to_string()).unwrap();
        let repo_name = base.make_relative(&url).unwrap();
        let status = match r.state.as_str() {
            "open" => "<svg style=\"color: #1a7f37; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
            aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M7.177 3.073L9.573.677A.25.25 0 0110 .854v4.792a.25.25 
            0 01-.427.177L7.177 3.427a.25.25 0 010-.354zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 
            113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zM11 2.5h-1V4h1a1 1 0 011 1v5.628a2.251 2.251 
            0 101.5 0V5A2.5 2.5 0 0011 2.5zm1 10.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0zM3.75 12a.75.75 0 100 1.5.75.75 0 
            000-1.5z\"></path></svg>".to_string(),
            "closed" => "<svg style=\"color: #8250df; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
             aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M5 3.254V3.25v.005a.75.75 0 110-.005v.004zm.45 1.9a2.25 2.25 
             0 10-1.95.218v5.256a2.25 2.25 0 101.5 0V7.123A5.735 5.735 0 009.25 9h1.378a2.251 2.251 0 100-1.5H9.25a4.25 4.25 0
              01-3.8-2.346zM12.75 9a.75.75 0 100-1.5.75.75 0 000 1.5zm-8.5 4.5a.75.75 0 100-1.5.75.75 0 000 1.5z\"></path></svg>".to_string(),
            _ => r.state.to_string()
        };

        TemplatePr {
            status,
            created_at: r.created_at.split('T').collect::<Vec<&str>>()[0].to_string(),
            title: r.title.to_string(),
            html_url: r.html_url.to_string(),
            repo_name,
            comments: r.comments,
            comments_present: (r.comments > 0, r.comments == 1),
            body: markdown::to_html(&r.body.to_string()),
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

pub fn generate_html_file(
    user: octocrab::models::User,
    prs: &[TemplatePr],
    reviews: &[TemplatePr],
    args: &crate::models::Args,
) -> Result<(), Box<dyn Error>> {
    let mut reg = Handlebars::new();
    static TEMPLATE: &str = include_str!("./template/template.hbs");
    reg.register_template_string("template", TEMPLATE).unwrap();
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
    data.insert("user".to_string(), to_json(user.login));
    data.insert("prs".to_string(), to_json(prs));
    data.insert("reviews".to_string(), to_json(reviews));
    data.insert("prs_len".to_string(), to_json(prs.len()));
    data.insert("reviews_len".to_string(), to_json(reviews.len()));
    data.insert("start_date".to_string(), to_json(from));
    data.insert("end_date".to_string(), to_json(to));

    let output_file_name = "self-assessment.html";
    let mut output_file = File::create(output_file_name)?;
    reg.render_to_write("template", &data, &mut output_file)?;

    println!(
        "[self-assessment] ✨ Generated a report containing {} PRs ({} authored, {} reviewed)",
        prs.len() + reviews.len(),
        prs.len(),
        reviews.len()
    );

    Ok(())
}
