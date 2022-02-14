use clap::Parser;
use serde::{Deserialize, Serialize};

pub enum GuardianPullRequests {
    AuthoredByMe,
    ReviewedByMe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubSearchResponse {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<GithubSearchResponseItem>,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct GithubSearchResponseItem {
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub title: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub state: String,
    pub assignee: Option<String>,
    pub milestone: Option<Milestone>,
    pub comments: u32,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub pull_request: PullRequest,
    pub body: String,
    pub score: f32,
    pub locked: bool,
    pub author_association: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub r#type: String,
    pub site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Label {
    pub id: u32,
    pub node_id: String,
    pub url: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub url: String,
    pub html_url: String,
    pub labels_url: String,
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub state: String,
    pub title: String,
    pub description: String,
    pub creator: User,
    pub open_issues: u32,
    pub closed_issues: u32,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: String,
    pub due_on: String,
}

#[derive(Serialize)]
pub struct TemplatePr {
    pub status: String,
    pub created_at: String,
    pub title: String,
    pub html_url: String,
    pub repo_name: String,
    pub comments: u32,
    pub comments_present: (bool, bool),
    pub body: String,
    pub labels: String,
    pub author: String,
    pub profile_pic: String,
}

// CLI struct
#[derive(Parser, Debug, Serialize)]
#[clap(author, version, about, long_about = None)]

pub struct Args {
    /// Match PRs that were created after this date
    #[clap(short, long, default_value = "*")]
    pub from: String,

    /// Match PRs that were created up until this date
    #[clap(short, long, default_value = "*")]
    pub to: String,

    /// Github authentication token. This is needed to run the CLI tool.
    /// You can get a personal access token at https://github.com/settings/tokens/new
    #[clap(short, long, default_value = "")]
    #[serde(rename = "auth-token")]
    pub auth_token: String,
}
