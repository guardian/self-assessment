use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug)]
pub enum GuardianPullRequests {
    AuthoredByMe,
    ReviewedByMe,
}

impl Display for GuardianPullRequests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            GuardianPullRequests::AuthoredByMe => write!(f, "pull requests authored by you"),
            GuardianPullRequests::ReviewedByMe => write!(f, "pull requests reviewed by you"),
        }
    }
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
    pub body: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TrelloBoard {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrelloCard {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub id_members: Vec<String>,
    pub url: String,
    pub date_last_activity: String,
    pub labels: Vec<TrelloLabel>,
}

#[derive(Serialize)]
pub struct TemplateTrelloCard {
    pub name: String,
    pub url: String,
    pub labels: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrelloLabel {
    pub id: String,
    pub id_board: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrelloUser {
    pub id: String,
    pub full_name: String,
    pub avatar_url: String,
}

#[derive(Serialize)]
pub struct BoardAndCards {
    pub board: String,
    pub cards: Vec<TemplateTrelloCard>,
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
