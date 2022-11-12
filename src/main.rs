pub mod cli;
pub mod credentials;
pub mod generate_report;
pub mod github;
pub mod models;
pub mod trello;

use crate::generate_report::generate_report;
use clap::StructOpt;
use cli::{Args, Commands};
use credentials::set_credentials;
use ini::Ini;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let credential_store_path = format!("{}/.selfassessment", shellexpand::tilde("~/"));
    let mut credential_store = match Ini::load_from_file(&credential_store_path) {
        Ok(ini) => ini,
        Err(_) => Ini::new(),
    };

    match args.command {
        Commands::Auth { token } => {
            set_credentials(
                &mut credential_store,
                credential_store_path,
                "GitHub".to_owned(),
                "GITHUB_TOKEN".to_owned(),
                token,
            )?;
        }
        Commands::TrelloAuth { key, token } => {
            set_credentials(
                &mut credential_store,
                credential_store_path.clone(),
                "Trello".to_owned(),
                "TRELLO_KEY".to_owned(),
                key,
            )?;

            set_credentials(
                &mut credential_store,
                credential_store_path,
                "Trello".to_owned(),
                "TRELLO_TOKEN".to_owned(),
                token,
            )?;
        }
        Commands::GenerateReport {
            from,
            to,
            skip_trello,
        } => {
            generate_report(from, to, skip_trello).await?;
        }
    }

    Ok(())
}
