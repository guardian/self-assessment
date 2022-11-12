use crate::models::*;
use chrono::DateTime;
use std::collections::HashMap;
use std::error::Error;

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
    from: &str,
    to: &str,
) -> anyhow::Result<HashMap<String, Vec<TrelloCard>>> {
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
                r#"https://api.trello.com/1/boards/{}/cards/all?key={}&token={}&fields=url,idMembers,name,desc,dateLastActivity,labels"#,
                board_id, &trello_key, &trello_token
            ))
            .send()
            .await?
            .json()
            .await?;

        // Only collect trello cards you're assigned to
        let my_cards_only: Vec<TrelloCard> = all_cards_in_board
            .into_iter()
            .filter(|card| card.id_members.contains(&trello_user.id))
            .filter(|card| trello_cards_date_range(card, from, to))
            .collect();

        if !my_cards_only.is_empty() {
            trello_cards.insert(board_name, my_cards_only);
        }
    }

    Ok(trello_cards)
}

pub fn trello_cards_date_range(card: &TrelloCard, from: &str, to: &str) -> bool {
    let from = match from {
        "*" => true,
        _ => {
            card.date_last_activity
                >= DateTime::parse_from_rfc3339(&format!("{}T00:00:00.00Z", from)).unwrap()
        }
    };

    let to = match to {
        "*" => true,
        _ => {
            card.date_last_activity
                <= DateTime::parse_from_rfc3339(&format!("{}T00:00:00.00Z", to)).unwrap()
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
