use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
        id: i32,
    },
    /// Runs some stuff for testing
    Test,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
            println!("Show info for id {}", id);

            #[derive(Serialize)]
            struct AnimeInfoVariables {
                id: i32,
            }

            #[derive(Serialize)]
            struct GraphQlRequest<'a> {
                query: &'a str,
                variables: AnimeInfoVariables,
            }

            const QUERY: &str = r#"
                query ($id: Int) {
                    Media (type: ANIME, id: $id) {
                        id
                        title {
                            romaji
                            native
                        }
                        season
                        seasonYear
                        episodes
                        averageScore
                        startDate {
                            year
                            month
                            day
                        }
                        endDate {
                            year
                            month
                            day
                        }
                    }
                }
                "#;

            let request = GraphQlRequest {
                query: QUERY,
                variables: AnimeInfoVariables { id },
            };

            let response: serde_json::Value = reqwest::Client::new()
                .post("https://graphql.anilist.co/")
                .json(&request)
                .send()
                .await?
                .json()
                .await?;

            println!("{response:#?}");
        }
        Commands::Test => {
            #[derive(Deserialize)]
            struct Ip {
                #[serde(rename = "origin")]
                ip_address: String,
            }

            let response: Ip = reqwest::get("https://httpbin.org/ip").await?.json().await?;

            println!("Your ip is {}", response.ip_address)
        }
    };

    Ok(())
}
