use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate to Github.
    /// This is needed for the CLI tool to access
    /// the Guardian's private repositories to which the user has access.
    /// You can get a personal access token at <https://github.com/settings/tokens/new>
    Auth {
        /// Github authentication token.
        token: String,
    },
    /// Authenticate to Trello. An API key and a server token are required.
    /// For more information, run
    /// self-assessment trello-auth --help
    TrelloAuth {
        /// Trello API key.
        /// You can get an API key at <https://trello.com/app-key>
        /// Note: you need to be logged into Trello to be able to see the page.
        /// Both the API key and the server token need to be set for the generated report
        /// to include Trello cards.
        key: String,
        /// Trello server token.
        /// You can get a server token by following the link at <https://trello.com/app-key>
        /// Both the API key and the server token need to be set for the generated report
        /// to include Trello cards.
        token: String,
    },
    /// Generate a report containing a list of PRs authored and reviewed by you,
    /// as well as an optional report of Trello boards and cards you are assigned to.
    /// For more information, run self-assessment generate-report --help
    GenerateReport {
        /// Match PRs and Trello cards that were created up until this date.
        /// The date must be in the YYYY-MM-DD format.
        #[clap(short, long, default_value = "*")]
        from: String,
        /// Match PRs and Trello cards that were created up until this date.
        /// The date must be in the YYYY-MM-DD format.
        #[clap(short, long, default_value = "*")]
        to: String,
        /// Skip Trello report.
        /// Passing this flag generates a report that does not include Trello cards.
        #[clap(short, long)]
        skip_trello: bool,
    },
}

#[derive(Debug)]
pub enum AuthType {
    GitHubAuthToken,
    TrelloApiKey,
    TrelloServerToken,
}
