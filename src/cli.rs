use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use image::DynamicImage;

use super::completion::generate_completions;
use super::display::{Style, print_anime_info, print_character_info};
use super::models::Anime;

use super::anilist::{
    fetch_anime_completion_search, fetch_anime_info, fetch_anime_search, fetch_character_info,
    fetch_image,
};

#[derive(Parser, Debug)]
#[command(version)]
/// Some anime cli
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
enum InfoType {
    #[value(alias = "a")]
    Anime,
    #[value(alias = "c")]
    Character,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show information about an anime or character
    Info {
        /// The AniList id
        id: i64,
        /// The type of information to retrieve from AniList
        #[arg(long, short, ignore_case = true, default_value = "anime")]
        r#type: InfoType,
    },
    /// Search for anime
    Search {
        /// The query to search for
        query: String,
        #[arg(long, short = 'p', default_value = "1")]
        /// The search page number
        page: i64,
        /// How many items to show per page
        #[arg(long, short = 'x', default_value = "8")]
        per_page: i64,
    },
    // FIXME: this hide true doesn't work for completions
    // https://github.com/clap-rs/clap/discussions/5214#discussioncomment-7577615
    #[command(hide = true)]
    /// Internal command for completions
    CompletionSearch { query: String },
    /// Generate zsh completions and print them to the screen
    GenerateZshCompletions,
    /// Test some stuff
    Test {
        #[arg(long, short, ignore_case = true, default_value = "anime")]
        r#type: InfoType,
    },
}

pub async fn parse_args() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id, r#type } => {
            let client = reqwest::Client::new();

            match r#type {
                InfoType::Anime => {
                    let anime = fetch_anime_info(&client, id).await?;

                    let cover_url = anime
                        .cover_image
                        .as_ref()
                        .and_then(|image| image.extra_large.as_deref());

                    let cover_image = fetch_image(&client, cover_url).await?;

                    print_anime_info(&anime, cover_image.as_ref(), Style::Large)?;
                }
                InfoType::Character => {
                    let character = fetch_character_info(&client, id).await?;

                    let image_url = character
                        .image
                        .as_ref()
                        .and_then(|image| image.large.as_deref());

                    let cover_image = fetch_image(&client, image_url).await?;

                    print_character_info(&character, cover_image.as_ref())?;
                }
            }
        }
        Commands::CompletionSearch { query } => {
            dbg!(&query);

            let client = reqwest::Client::new();
            let completions = fetch_anime_completion_search(&client, query).await?;

            for completion in completions {
                println!("{completion}")
            }
        }
        Commands::GenerateZshCompletions => {
            let mut cmd = Cli::command();

            let zsh_completion = generate_completions(&mut cmd)?;

            println!("{zsh_completion}")
        }
        Commands::Search {
            page,
            query,
            per_page,
        } => {
            let client = reqwest::Client::new();
            let results = fetch_anime_search(&client, Some(query), page, per_page).await?;

            let items: Vec<(Anime, Option<DynamicImage>)> =
                futures::future::join_all(results.items.into_iter().map(|anime| async {
                    let url = anime
                        .cover_image
                        .as_ref()
                        .and_then(|image| image.medium.as_deref());

                    let cover = fetch_image(&client, url).await.ok().flatten();

                    (anime, cover)
                }))
                .await;

            for (anime, cover) in items {
                print_anime_info(&anime, cover.as_ref(), Style::Small)?;

                println!();
            }

            if results.has_next_page {
                println!("next page is available")
            }
        }
        Commands::Test { r#type } => {
            dbg!(r#type);
        }
    }

    Ok(())
}
