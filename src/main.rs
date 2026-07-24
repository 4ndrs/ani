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
    /// Test running stuff
    Test,
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

            print_anime_info(&anime);
        }
        Commands::Test => {
            println!("testing");

            let url = "https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/bx192800-r8zlO0VY0jJP.jpg";

            let bytes = reqwest::get(url).await?.bytes().await?;

            let img = image::load_from_memory(&bytes)?;

            let config = viuer::Config {
                height: Some(20),
                absolute_offset: false,
                ..Default::default()
            };

            viuer::print(&img, &config)?;
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

    if let Some(errors) = response.errors {
        let messages = errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
            .join("; ");

        return Err(Error::other(format!("GraphQL error: {messages}")).into());
    }

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
        let format = match self {
            anime_info::MediaFormat::TV => "TV",
            anime_info::MediaFormat::OVA => "OVA",
            anime_info::MediaFormat::ONA => "ONA",
            anime_info::MediaFormat::MANGA => "Manga",
            anime_info::MediaFormat::MOVIE => "Movie",
            anime_info::MediaFormat::NOVEL => "Novel",
            anime_info::MediaFormat::MUSIC => "Music",
            anime_info::MediaFormat::SPECIAL => "Special",
            anime_info::MediaFormat::ONE_SHOT => "One Shot",
            anime_info::MediaFormat::TV_SHORT => "TV Short",
            _ => "Unknown",
        };

        formatter.write_str(format)
    }
}

impl Display for anime_info::MediaSeason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let season = match self {
            anime_info::MediaSeason::FALL => "Fall",
            anime_info::MediaSeason::WINTER => "Winter",
            anime_info::MediaSeason::SPRING => "Spring",
            anime_info::MediaSeason::SUMMER => "Summer",
            _ => "Unknown",
        };

        formatter.write_str(season)
    }
}

fn print_anime_info(media: &anime_info::AnimeInfoMedia) {
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

    let romaji = media
        .title
        .as_ref()
        .and_then(|title| title.romaji.as_deref());

    let native = media
        .title
        .as_ref()
        .and_then(|title| title.native.as_deref());

    match (romaji, native) {
        (Some(romaji), Some(native)) => {
            println!("{romaji}");
            println!("{native}");
        }
        (Some(romaji), None) => println!("{romaji}"),
        (None, Some(native)) => println!("{native}"),
        (None, None) => println!("No Title"),
    }

    println!();

    let mut details = Vec::new();

    if let Some(format) = &media.format {
        details.push(format.to_string());
    }

    if let Some(episodes) = media.episodes {
        let label = if episodes == 1 { "episode" } else { "episodes" };

        details.push(format!("{episodes} {label}"));
    }

    if let Some(season_year) = media.season_year {
        details.push(season_year.to_string());
    }

    if !details.is_empty() {
        println!("{}", details.join(" · "))
    }

    if let (Some(season), Some(year)) = (&media.season, media.season_year) {
        println!("{season} {year}");
    }

    if let Some(score) = media.average_score {
        println!("Score: {score}/100");
    }

    if let Some(genres) = &media.genres {
        let genres: Vec<&str> = genres.iter().flatten().map(String::as_str).collect();

        if !genres.is_empty() {
            println!("Genres: {}", genres.join(" · "))
        };
    }

    println!();

    if let Some(description) = &media.description {
        println!("{description}\n");
    }

    if let Some(site_url) = &media.site_url {
        println!("AniList: {site_url}")
    }
}
