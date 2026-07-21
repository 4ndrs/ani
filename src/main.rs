use std::{
    fmt::{self, Display},
    io::Error,
};

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
            let client = reqwest::Client::new();

            let anime = fetch_anime_info(&client, id).await?;

            print_anime_info(anime);
        }
    };

    Ok(())
}

async fn fetch_anime_info(
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

impl Display for anime_info::MediaFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Display for anime_info::MediaSeason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

fn print_anime_info(media: anime_info::AnimeInfoMedia) {
    // Made in Abyss
    // メイドインアビス
    //
    // TV · 13 episodes · 2017
    // Summer 2017
    // Score: 86/100
    // Genres: Adventure · Drama · Fantasy · Mystery · Sci-Fi
    //
    // An enormous cave system known as the Abyss is the last unexplored
    // place in the world...
    //
    // AniList: https://anilist.co/anime/97986

    if let Some(title_romaji) = media.title.as_ref().and_then(|title| title.romaji.as_ref())
        && let Some(title_native) = media.title.as_ref().and_then(|title| title.native.as_ref())
    {
        println!("{title_romaji}");
        println!("{title_native}");
    } else if let Some(title_romaji) = media.title.as_ref().and_then(|title| title.romaji.as_ref())
    {
        println!("{title_romaji}")
    } else if let Some(title_native) = media.title.and_then(|title| title.native) {
        println!("{title_native}")
    } else {
        println!("No Title")
    }

    println!();

    let mut third_line: Vec<String> = vec![];

    if let Some(format) = media.format {
        third_line.push(format.to_string().replace("_", " "));
    };

    if let Some(episodes) = media.episodes {
        let mut text = "episodes";

        if episodes == 1 {
            text = "episode"
        }

        third_line.push(episodes.to_string() + " " + text);
    }

    if let Some(season_year) = media.season_year.as_ref() {
        third_line.push(season_year.to_string());
    }

    print_tags(third_line);

    if let Some(season) = media.season
        && let Some(season_year) = media.season_year
    {
        println!("{season} {season_year}");
    }

    if let Some(score) = media.average_score {
        println!("Score: {score}/100");
    }

    if let Some(genres) = media.genres {
        print!("Genres: ");

        let genres = genres.into_iter().flatten().collect();

        print_tags(genres);
    }

    println!();

    if let Some(description) = media.description {
        println!("{description}\n");
    }

    if let Some(site_url) = media.site_url {
        println!("AniList: {site_url}")
    }
}

fn print_tags(tags: Vec<String>) {
    let len = tags.len();

    for (index, tag) in tags.iter().enumerate() {
        print!("{tag}");

        if index != len - 1 {
            print!(" · ");
        }
    }

    println!();
}
