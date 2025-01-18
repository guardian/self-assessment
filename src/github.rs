use std::{borrow::Cow, collections::HashMap, process};

use colorsys::{Hsl, Rgb};
use octocrab::Octocrab;
use reqwest::Url;

use crate::models::{
    GithubSearchResponse, GithubSearchResponseItem, GuardianPullRequests, TemplatePr,
};

const GITHUB_ORG: &str = "guardian";

const OPEN_PR: &str = "<svg style=\"color: #1a7f37; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M7.177 3.073L9.573.677A.25.25 0 0110 .854v4.792a.25.25 
0 01-.427.177L7.177 3.427a.25.25 0 010-.354zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 
113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zM11 2.5h-1V4h1a1 1 0 011 1v5.628a2.251 2.251 
0 101.5 0V5A2.5 2.5 0 0011 2.5zm1 10.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0zM3.75 12a.75.75 0 100 1.5.75.75 0 
000-1.5z\"></path></svg>";

const MERGED_PR: &str = "<svg style=\"color: #8250df; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M5 3.254V3.25v.005a.75.75 0 110-.005v.004zm.45 1.9a2.25 2.25 
0 10-1.95.218v5.256a2.25 2.25 0 101.5 0V7.123A5.735 5.735 0 009.25 9h1.378a2.251 2.251 0 100-1.5H9.25a4.25 4.25 0
 01-3.8-2.346zM12.75 9a.75.75 0 100-1.5.75.75 0 000 1.5zm-8.5 4.5a.75.75 0 100-1.5.75.75 0 000 1.5z\"></path></svg>";

const CLOSED_PR: &str = "<svg style=\"color: #d1242f; margin-left:10px;\" viewBox=\"0 0 16 16\" version=\"1.1\" width=\"16\" height=\"16\" 
aria-hidden=\"true\"><path fill=\"currentColor\" d=\"M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 
0V5.372A2.25 2.25 0 0 1 1.5 3.25Zm5.677-.177L9.573.677A.25.25 0 0 1 10 .854V2.5h1A2.5 2.5 0 0 1 13.5 5v5.628a2.251 
2.251 0 1 1-1.5 0V5a1 1 0 0 0-1-1h-1v1.646a.25.25 0 0 1-.427.177L7.177 3.427a.25.25 0 0 1 0-.354ZM3.75 2.5a.75.75 
0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm0 9.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm8.25.75a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Z\"></path></svg>";

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
    let lightness_switch = ((perceived_lightness - 0.453) * -1000.0).clamp(0.0, 1.0);
    let hsl = Hsl::new(0.0, 0.0, lightness_switch * 100.0, Some(1.0));
    hsl.to_css_string()
}

pub async fn search_pull_requests<'a>(
    client: &Octocrab,
    pr_type: GuardianPullRequests,
    params: &mut HashMap<&'static str, Cow<'a, str>>,
    from: &str,
    to: &str,
) -> anyhow::Result<Vec<GithubSearchResponseItem>> {
    let mut all_results: Vec<GithubSearchResponseItem> = vec![];
    let mut count = 1;

    match pr_type {
        GuardianPullRequests::AuthoredByMe => {
            params.insert(
                "q",
                Cow::from(format!(
                    "org:{} author:@me is:pr created:{}..{}",
                    GITHUB_ORG, from, to
                )),
            );
        }
        GuardianPullRequests::ReviewedByMe => {
            params.insert(
                "q",
                Cow::from(format!(
                    "org:{} -author:@me reviewed-by:@me is:pr created:{}..{}",
                    GITHUB_ORG, from, to
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

    Ok(all_results)
}

pub fn format_prs(results: &[GithubSearchResponseItem]) -> Vec<TemplatePr> {
    let base = Url::parse(&format!("https://api.github.com/repos/{}/", GITHUB_ORG)).unwrap();

    results
    .iter()
    .map(|r| {
        let url = Url::parse(&r.repository_url.to_string()).unwrap();
        let repo_name = base.make_relative(&url).unwrap();
        let status = match r.state.as_str() {
            "open" => OPEN_PR.to_string(),
            "closed" => if r.pull_request.merged_at.is_some() { MERGED_PR.to_string() } else { CLOSED_PR.to_string() },
            _ => r.state.to_string(),
        };

        TemplatePr {
            status,
            created_at: r.created_at.format("%Y-%m-%d").to_string(),
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
