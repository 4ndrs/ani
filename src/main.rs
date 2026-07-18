use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version)]
/// Some anime cli
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show information about an anime
    Info {
        /// The AniList anime id
        id: u32,
    },
    /// Runs some stuff for testing
    Test,
}

#[derive(Deserialize)]
struct Ip {
    #[serde(rename = "origin")]
    ip_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
            println!("Show info for id {}", id)
        }
        Commands::Test => {
            let response = reqwest::get("https://httpbin.org/ip")
                .await?
                .json::<Ip>()
                .await?;

            println!("Your ip is {}", response.ip_address)
        }
    };

    Ok(())
}
