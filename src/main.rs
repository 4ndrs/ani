use std::io::Error;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
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
                        description
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

            #[derive(Deserialize, Debug)]
            struct FuzzyDate {
                day: Option<i32>,
                month: Option<i32>,
                year: Option<i32>,
            }

            #[derive(Deserialize, Debug)]
            #[serde(rename_all = "UPPERCASE")]
            enum Season {
                Spring,
                Summer,
                Fall,
                Winter,
            }

            #[derive(Deserialize, Debug)]
            struct Title {
                romaji: Option<String>,
                native: Option<String>,
            }

            #[derive(Deserialize, Debug)]
            struct Media {
                id: i32,
                title: Option<Title>,
                description: Option<String>,

                #[serde(rename = "averageScore")]
                average_score: Option<i32>,

                #[serde(rename = "startDate")]
                start_date: Option<FuzzyDate>,

                #[serde(rename = "endDate")]
                end_date: Option<FuzzyDate>,

                season: Option<Season>,

                #[serde(rename = "seasonYear")]
                season_year: Option<i32>,
            }

            #[derive(Deserialize, Debug)]
            struct GraphQLData {
                #[serde(rename = "Media")]
                media: Option<Media>,
            }

            #[derive(Deserialize, Debug)]
            struct GraphQLError {
                message: String,
            }

            #[derive(Deserialize, Debug)]
            struct GraphQLResponse {
                data: Option<GraphQLData>,
                errors: Option<Vec<GraphQLError>>,
            }

            println!("Show info for id {}", id);

            let request = GraphQlRequest {
                query: QUERY,
                variables: AnimeInfoVariables { id },
            };

            let response: GraphQLResponse = reqwest::Client::new()
                .post("https://graphql.anilist.co/")
                .json(&request)
                .send()
                .await?
                .json()
                .await?;

            println!("{response:#?}\n");

            let data = response
                .data
                .ok_or_else(|| Error::other("response contained no data"))?;

            let media = data
                .media
                .ok_or_else(|| Error::other("response contained no media"))?;

            let id = media.id;

            let title = media
                .title
                .and_then(|title| title.native)
                .unwrap_or_else(|| "No title".to_owned());

            let description = media
                .description
                .unwrap_or_else(|| "No description".to_owned());

            println!("Id: {id}");
            println!("Title: {title}");
            println!("Description: {description}");
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
