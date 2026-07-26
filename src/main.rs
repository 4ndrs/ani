use std::{
    fmt::{self, Display},
    io::{Error, Write, stdout},
};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use crossterm::{cursor, execute, terminal};
use graphql_client::{GraphQLQuery, Response};
use image::DynamicImage;

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
    // FIXME: this hide true doesn't work for completions
    // https://github.com/clap-rs/clap/discussions/5214#discussioncomment-7577615
    #[command(hide = true)]
    /// Internal command for completions
    CompletionSearch { query: String },
    /// Generate zsh completions and print them to the screen
    GenerateZshCompletions,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_info.graphql",
    response_derives = "Debug"
)]
struct AnimeInfo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_completion_search.graphql",
    response_derives = "Debug"
)]
struct AnimeCompletionSearch;

const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co/";

const ANIME_INFO_ZSH_COMPLETION: &str = r#"_ani_info() {
    # a previous completion call already produced and displayed a list
    # reuse it and begin selecting the items
    if [[ "${compstate[old_list]}" == "shown" ]]; then
        compstate[old_list]=keep
        compstate[insert]=menu
        return
    fi

    local -a ids
    local -a displays

    local line id title

    local query="${words[CURRENT]}"

    if (( ${#query} < 2 )); then
        _message 'Type at least 2 characters to search AniList'
        return
    fi

    zle -M "Searching AniList for '$query'..."

    lines=("${(@f)$(ani completion-search "$query" 2>/dev/null)}")

    for line in "${lines[@]}"; do
        id="${line%%$'\t'*}"
        title="${line#*$'\t'}"

        ids+=("$id")
        displays+=("${(r:8:)id} $title")
    done

    compadd -U -Q -d displays -a ids

    # first tab shows the list but doesn't modify the command line
    compstate[list]='list force'
    compstate[insert]=''
}
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
            let client = reqwest::Client::new();

            let anime = fetch_anime_info(&client, id).await?;

            let image_url = anime
                .cover_image
                .as_ref()
                .and_then(|image| image.extra_large.as_deref());

            let cover_image = fetch_image(&client, image_url).await?;

            print_anime_info(&anime, cover_image.as_ref())?;
        }
        Commands::CompletionSearch { query } => {
            dbg!(&query);

            std::thread::sleep(std::time::Duration::from_secs(5));

            let client = reqwest::Client::new();
            let results = fetch_anime_completion_search(&client, query).await?;

            let results: Vec<_> = results
                .into_iter()
                .map(|anime| {
                    let id = anime.id;

                    let title = anime
                        .title
                        .and_then(|title| title.romaji)
                        .unwrap_or_else(|| "No Title".to_string());

                    format!("{id}\t{title}")
                })
                .collect();

            for anime in results {
                println!("{anime}");
            }
        }
        Commands::GenerateZshCompletions => {
            let mut cmd = Cli::command();
            let mut buffer: Vec<u8> = Vec::new();

            let name = cmd.get_name().to_string();

            generate(shells::Zsh, &mut cmd, name, &mut buffer);

            let zsh_completion = String::from_utf8(buffer)?;
            let zsh_completion = zsh_completion.replace("anime id:_default", "anime id:_ani_info");
            let zsh_completion = format!("{}\n{}", zsh_completion, ANIME_INFO_ZSH_COMPLETION);

            println!("{zsh_completion}")
        }
    }

    Ok(())
}

async fn fetch_image(
    client: &reqwest::Client,
    url: Option<&str>,
) -> Result<Option<DynamicImage>, Box<dyn std::error::Error>> {
    let Some(url) = url else { return Ok(None) };

    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let image = image::load_from_memory(&bytes)?;

    Ok(Some(image))
}

async fn fetch_anime_completion_search(
    client: &reqwest::Client,
    query: String,
) -> Result<
    Vec<anime_completion_search::AnimeCompletionSearchPageResults>,
    Box<dyn std::error::Error>,
> {
    let variables = anime_completion_search::Variables { query };
    let request_body = AnimeCompletionSearch::build_query(variables);

    let response: Response<anime_completion_search::ResponseData> = client
        .post(ANILIST_GRAPHQL_URL)
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

    let results = response
        .data
        .ok_or_else(|| Error::other("response contained no data"))?
        .page
        .ok_or_else(|| Error::other("response contained no page"))?
        .results
        .ok_or_else(|| Error::other("response contained no results"))?;

    let results: Vec<_> = results.into_iter().flatten().collect();

    Ok(results)
}

async fn fetch_anime_info(
    client: &reqwest::Client,
    id: i64,
) -> Result<anime_info::AnimeInfoMedia, Box<dyn std::error::Error>> {
    let variables = anime_info::Variables { id };
    let request_body = AnimeInfo::build_query(variables);

    let response: Response<anime_info::ResponseData> = client
        .post(ANILIST_GRAPHQL_URL)
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

impl Display for anime_info::MediaStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            anime_info::MediaStatus::HIATUS => "Hiatus",
            anime_info::MediaStatus::FINISHED => "Finished",
            anime_info::MediaStatus::RELEASING => "Releasing",
            anime_info::MediaStatus::CANCELLED => "Cancelled",
            anime_info::MediaStatus::NOT_YET_RELEASED => "Not Yet Released",
            _ => "Unknown",
        };

        formatter.write_str(status)
    }
}

const COVER_WIDTH: u32 = 27;

fn print_anime_info(
    media: &anime_info::AnimeInfoMedia,
    cover: Option<&DynamicImage>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ┌──────────────────────┐  Made in Abyss
    // │                      │  メイドインアビス
    // │                      │
    // │                      │  TV · 13 episodes · 2017
    // │                      │  Summer 2017
    // │                      │  Score: 86/100
    // │                      │  Status: Finished
    // │     COVER IMAGE      │  Genres: Adventure · Drama · Fantasy · Mystery · Sci-Fi
    // │                      │
    // │                      │  An enormous cave system known as the Abyss is the last unexplored
    // │                      │  place in the world...
    // │                      │
    // │                      │  AniList: https://anilist.co/anime/97986
    // │                      │
    // └──────────────────────┘

    let mut stdout = stdout();
    let mut shift_right = 0;
    let mut row_below_image = 0;

    let (terminal_columns, terminal_rows) = terminal::size()?;

    if let Some(cover) = cover {
        let cover_resized = viuer::resize(cover, Some(COVER_WIDTH), None);

        let (cover_columns, cover_rows) = (
            u16::try_from(cover_resized.width())?,
            // this is actually x2 the terminal height (each cell contains 2 pixels)
            u16::try_from(cover_resized.height().div_ceil(2))?,
        );

        let (_, current_row) = cursor::position()?;

        let available_rows = terminal_rows.saturating_sub(current_row).saturating_sub(1);

        let needs_scrolling = available_rows < cover_rows;

        if needs_scrolling {
            // reserve space for the image (scrolling breaks text positioning)
            let rows_to_scroll = cover_rows - available_rows;

            execute!(
                stdout,
                terminal::ScrollUp(rows_to_scroll),
                cursor::MoveUp(rows_to_scroll)
            )?;
        }

        let (_, current_row) = cursor::position()?;

        let config = viuer::Config {
            x: 1,
            width: Some(COVER_WIDTH),
            restore_cursor: true,
            absolute_offset: false,
            ..Default::default()
        };

        viuer::print(cover, &config)?;

        row_below_image = current_row.saturating_add(cover_rows);
        shift_right = cover_columns + 3;
    }

    let romaji = media
        .title
        .as_ref()
        .and_then(|title| title.romaji.as_deref());

    let native = media
        .title
        .as_ref()
        .and_then(|title| title.native.as_deref());

    execute!(stdout, cursor::MoveToColumn(shift_right))?;

    match (romaji, native) {
        (Some(romaji), Some(native)) => {
            if native == romaji {
                writeln!(stdout, "{native}")?;
            } else {
                writeln!(stdout, "{romaji}")?;
                execute!(stdout, cursor::MoveToColumn(shift_right))?;
                writeln!(stdout, "{native}")?;
            }
        }
        (Some(romaji), None) => writeln!(stdout, "{romaji}")?,
        (None, Some(native)) => writeln!(stdout, "{native}")?,
        (None, None) => writeln!(stdout, "No Title")?,
    }

    writeln!(stdout)?;

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
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "{}", details.join(" · "))?
    }

    if let (Some(season), Some(year)) = (&media.season, media.season_year) {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "{season} {year}")?;
    }

    if let Some(score) = media.average_score {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "Score: {score}/100")?;
    }

    if let Some(status) = &media.status {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "Status: {status}")?
    }

    let space_available = usize::from(
        terminal_columns
            .saturating_sub(shift_right)
            .saturating_sub(1),
    );

    if let Some(genres) = &media.genres {
        let genres: Vec<&str> = genres.iter().flatten().map(String::as_str).collect();

        if !genres.is_empty() {
            let genres = format!("Genres: {}", genres.join(" · "));

            let lines = textwrap::wrap(&genres, space_available);

            for line in lines {
                execute!(stdout, cursor::MoveToColumn(shift_right))?;
                writeln!(stdout, "{line}")?;
            }
        };
    }

    writeln!(stdout)?;

    if let Some(description) = &media.description {
        let description: String = scraper::Html::parse_fragment(description)
            .root_element()
            .text()
            .collect();

        let lines = textwrap::wrap(&description, space_available);

        for line in lines {
            execute!(stdout, cursor::MoveToColumn(shift_right))?;
            writeln!(stdout, "{line}")?;
        }

        writeln!(stdout)?;
    }

    if let Some(site_url) = &media.site_url {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "AniList: {site_url}")?;
    }

    let (_, current_row) = cursor::position()?;

    if current_row < row_below_image {
        execute!(stdout, cursor::MoveTo(0, row_below_image))?
    }

    Ok(())
}
