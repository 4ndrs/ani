use std::io::Error;

use clap::{Parser, Subcommand};
use graphql_client::{GraphQLQuery, Response};

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
        id: i64,
    },
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_info.graphql",
    response_derives = "Debug"
)]
struct AnimeInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
            println!("Show info for id {id}");

            let client = reqwest::Client::new();

            let anime = fetch_anime(&client, id).await?;

            let id = anime.id;

            let title = anime
                .title
                .and_then(|title| title.native)
                .unwrap_or_else(|| "No title".to_owned());

            let description = anime
                .description
                .unwrap_or_else(|| "No description".to_owned());

            println!("Id: {id}");
            println!("Title: {title}");
            println!("Description: {description}");
        }
    };

    Ok(())
}

async fn fetch_anime(
    client: &reqwest::Client,
    id: i64,
) -> Result<anime_info::AnimeInfoMedia, Box<dyn std::error::Error>> {
    let variables = anime_info::Variables { id };
    let request_body = AnimeInfo::build_query(variables);

    let response: Response<anime_info::ResponseData> = client
        .post("https://graphql.anilist.co/")
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let data = response
        .data
        .ok_or_else(|| Error::other("response contained no data"))?;

    let media = data
        .media
        .ok_or_else(|| Error::other("response contained no media"))?;

    Ok(media)
}
