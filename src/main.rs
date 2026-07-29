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
    /// Search for anime
    Search {
        /// The query to search for
        query: String,
    },
    // FIXME: this hide true doesn't work for completions
    // https://github.com/clap-rs/clap/discussions/5214#discussioncomment-7577615
    #[command(hide = true)]
    /// Internal command for completions
    CompletionSearch { query: String },
    /// Generate zsh completions and print them to the screen
    GenerateZshCompletions,
    /// Test some stuff
    Test,
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_search.graphql",
    response_derives = "Debug"
)]
struct AnimeSearch;

struct AnimeCompletion {
    id: i64,
    title: String,
}

impl Display for AnimeCompletion {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}\t{}", self.id, self.title)
    }
}

struct AnimeTitle {
    romaji: Option<String>,
    native: Option<String>,
}

impl From<anime_info::AnimeInfoMediaTitle> for AnimeTitle {
    fn from(value: anime_info::AnimeInfoMediaTitle) -> Self {
        Self {
            romaji: value.romaji,
            native: value.native,
        }
    }
}

impl From<anime_search::AnimeSearchPageResultsTitle> for AnimeTitle {
    fn from(value: anime_search::AnimeSearchPageResultsTitle) -> Self {
        Self {
            romaji: value.romaji,
            native: value.native,
        }
    }
}

struct Anime {
    id: i64,
    title: Option<AnimeTitle>,
    genres: Option<Vec<String>>,
    format: Option<MediaFormat>,
    status: Option<MediaStatus>,
    season: Option<MediaSeason>,
    episodes: Option<i64>,
    site_url: Option<String>,
    cover_url: Option<String>,
    season_year: Option<i64>,
    description: Option<String>,
    average_score: Option<i64>,
}

impl From<anime_info::AnimeInfoMedia> for Anime {
    fn from(anime: anime_info::AnimeInfoMedia) -> Self {
        Self {
            id: anime.id,
            season_year: anime.season_year,
            average_score: anime.average_score,
            description: anime.description,
            site_url: anime.site_url,
            episodes: anime.episodes,
            genres: anime.genres.into_iter().flatten().collect(),
            status: anime.status.map(|status| status.into()),
            season: anime.season.map(|season| season.into()),
            format: anime.format.map(|format| format.into()),
            title: anime.title.map(|title| title.into()),
            cover_url: anime.cover_image.and_then(|cover| cover.extra_large),
        }
    }
}

impl From<anime_search::AnimeSearchPageResults> for Anime {
    fn from(anime: anime_search::AnimeSearchPageResults) -> Self {
        Self {
            id: anime.id,
            season_year: anime.season_year,
            average_score: anime.average_score,
            description: anime.description,
            site_url: anime.site_url,
            episodes: anime.episodes,
            genres: anime.genres.into_iter().flatten().collect(),
            status: anime.status.map(|status| status.into()),
            season: anime.season.map(|season| season.into()),
            format: anime.format.map(|format| format.into()),
            title: anime.title.map(|title| title.into()),
            cover_url: anime.cover_image.and_then(|cover| cover.extra_large),
        }
    }
}

struct AnimeSearchResults {
    has_next_page: bool,
    items: Vec<Anime>,
}

enum MediaSeason {
    FALL,
    WINTER,
    SPRING,
    SUMMER,

    #[allow(dead_code)]
    Other(String),
}

impl Display for MediaSeason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let season = match self {
            MediaSeason::FALL => "Fall",
            MediaSeason::WINTER => "Winter",
            MediaSeason::SPRING => "Spring",
            MediaSeason::SUMMER => "Summer",
            _ => "Unknown",
        };

        formatter.write_str(season)
    }
}

impl From<anime_info::MediaSeason> for MediaSeason {
    fn from(value: anime_info::MediaSeason) -> Self {
        match value {
            anime_info::MediaSeason::FALL => Self::FALL,
            anime_info::MediaSeason::WINTER => Self::WINTER,
            anime_info::MediaSeason::SPRING => Self::SPRING,
            anime_info::MediaSeason::SUMMER => Self::SUMMER,
            anime_info::MediaSeason::Other(other) => Self::Other(other),
        }
    }
}

impl From<anime_search::MediaSeason> for MediaSeason {
    fn from(value: anime_search::MediaSeason) -> Self {
        match value {
            anime_search::MediaSeason::FALL => Self::FALL,
            anime_search::MediaSeason::WINTER => Self::WINTER,
            anime_search::MediaSeason::SPRING => Self::SPRING,
            anime_search::MediaSeason::SUMMER => Self::SUMMER,
            anime_search::MediaSeason::Other(other) => Self::Other(other),
        }
    }
}

enum MediaFormat {
    TV,
    OVA,
    ONA,
    MANGA,
    MOVIE,
    NOVEL,
    MUSIC,
    SPECIAL,
    OneShot,
    TvShort,

    #[allow(dead_code)]
    Other(String),
}

impl Display for MediaFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let format = match self {
            MediaFormat::TV => "TV",
            MediaFormat::OVA => "OVA",
            MediaFormat::ONA => "ONA",
            MediaFormat::MANGA => "Manga",
            MediaFormat::MOVIE => "Movie",
            MediaFormat::NOVEL => "Novel",
            MediaFormat::MUSIC => "Music",
            MediaFormat::SPECIAL => "Special",
            MediaFormat::OneShot => "One Shot",
            MediaFormat::TvShort => "TV Short",
            _ => "Unknown",
        };

        formatter.write_str(format)
    }
}

impl From<anime_info::MediaFormat> for MediaFormat {
    fn from(value: anime_info::MediaFormat) -> Self {
        match value {
            anime_info::MediaFormat::TV => Self::TV,
            anime_info::MediaFormat::OVA => Self::OVA,
            anime_info::MediaFormat::ONA => Self::ONA,
            anime_info::MediaFormat::MOVIE => Self::MOVIE,
            anime_info::MediaFormat::MUSIC => Self::MUSIC,
            anime_info::MediaFormat::MANGA => Self::MANGA,
            anime_info::MediaFormat::NOVEL => Self::NOVEL,
            anime_info::MediaFormat::SPECIAL => Self::SPECIAL,
            anime_info::MediaFormat::TV_SHORT => Self::TvShort,
            anime_info::MediaFormat::ONE_SHOT => Self::OneShot,
            anime_info::MediaFormat::Other(other) => Self::Other(other),
        }
    }
}

impl From<anime_search::MediaFormat> for MediaFormat {
    fn from(value: anime_search::MediaFormat) -> Self {
        match value {
            anime_search::MediaFormat::TV => Self::TV,
            anime_search::MediaFormat::OVA => Self::OVA,
            anime_search::MediaFormat::ONA => Self::ONA,
            anime_search::MediaFormat::MOVIE => Self::MOVIE,
            anime_search::MediaFormat::MUSIC => Self::MUSIC,
            anime_search::MediaFormat::MANGA => Self::MANGA,
            anime_search::MediaFormat::NOVEL => Self::NOVEL,
            anime_search::MediaFormat::SPECIAL => Self::SPECIAL,
            anime_search::MediaFormat::TV_SHORT => Self::TvShort,
            anime_search::MediaFormat::ONE_SHOT => Self::OneShot,
            anime_search::MediaFormat::Other(other) => Self::Other(other),
        }
    }
}

enum MediaStatus {
    HIATUS,
    FINISHED,
    RELEASING,
    CANCELLED,
    NotYetReleased,

    #[allow(dead_code)]
    Other(String),
}

impl Display for MediaStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            MediaStatus::HIATUS => "Hiatus",
            MediaStatus::FINISHED => "Finished",
            MediaStatus::RELEASING => "Releasing",
            MediaStatus::CANCELLED => "Cancelled",
            MediaStatus::NotYetReleased => "Not Yet Released",
            _ => "Unknown",
        };

        formatter.write_str(status)
    }
}

impl From<anime_info::MediaStatus> for MediaStatus {
    fn from(value: anime_info::MediaStatus) -> Self {
        match value {
            anime_info::MediaStatus::HIATUS => Self::HIATUS,
            anime_info::MediaStatus::FINISHED => Self::FINISHED,
            anime_info::MediaStatus::RELEASING => Self::RELEASING,
            anime_info::MediaStatus::CANCELLED => Self::CANCELLED,
            anime_info::MediaStatus::Other(other) => Self::Other(other),
            anime_info::MediaStatus::NOT_YET_RELEASED => Self::NotYetReleased,
        }
    }
}

impl From<anime_search::MediaStatus> for MediaStatus {
    fn from(value: anime_search::MediaStatus) -> Self {
        match value {
            anime_search::MediaStatus::HIATUS => Self::HIATUS,
            anime_search::MediaStatus::FINISHED => Self::FINISHED,
            anime_search::MediaStatus::RELEASING => Self::RELEASING,
            anime_search::MediaStatus::CANCELLED => Self::CANCELLED,
            anime_search::MediaStatus::Other(other) => Self::Other(other),
            anime_search::MediaStatus::NOT_YET_RELEASED => Self::NotYetReleased,
        }
    }
}

const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co/";

const ANIME_INFO_ZSH_COMPLETION: &str = r#"_ani_info() {
    # a previous completion call already produced and displayed a list
    # reuse it and begin selecting the items
    if [[ "${compstate[old_list]}" == "shown" ]]; then
        compstate[old_list]=keep
        compstate[insert]=menu
        return
    fi

    local -a ids displays lines

    local line id title output

    local query="${words[CURRENT]}"

    if (( ${#query} < 2 )); then
        _message 'Type at least 2 characters to search AniList'
        return
    fi

    zle -M "Searching AniList for '$query'..."

    output="$(ani completion-search "$query" 2>/dev/null)"

    if [[ -z "$output" ]]; then
        _message "No anime found for '$query'"
        return
    fi

    lines=("${(f)output}")

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
            let cover_image = fetch_image(&client, anime.cover_url.as_deref()).await?;

            print_anime_info(&anime, cover_image.as_ref())?;
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
            let mut buffer: Vec<u8> = Vec::new();

            let name = cmd.get_name().to_string();

            generate(shells::Zsh, &mut cmd, name, &mut buffer);

            let zsh_completion =
                String::from_utf8(buffer)?.replace("anime id:_default", "anime id:_ani_info");

            let zsh_completion = format!("{}\n{}", zsh_completion, ANIME_INFO_ZSH_COMPLETION);

            println!("{zsh_completion}")
        }
        Commands::Search { query } => {
            let client = reqwest::Client::new();
            let results = fetch_anime_search(&client, Some(query), 1, 8).await?;

            print_anime_search(&client, results).await?;
        }
        Commands::Test => {
            let padding = 1;
            let cover_width = 20;
            let number_of_items = 10;
            let minimum_gap = 2;

            let mut stdout = stdout();

            let (terminal_columns, _) = crossterm::terminal::size()?;

            let space_available = terminal_columns.saturating_sub(padding * 2);

            let cover_width = u16::try_from(cover_width)?;

            let item_count = (space_available + minimum_gap) / (cover_width + minimum_gap);

            let gap = if item_count > 1 {
                (space_available - item_count * cover_width) / (item_count - 1)
            } else {
                0
            };

            dbg!(
                gap,
                padding,
                item_count,
                minimum_gap,
                cover_width,
                number_of_items,
                terminal_columns,
                space_available
            );

            let url = "https://s4.anilist.co/file/anilistcdn/media/anime/cover/small/bx197868-sm5jcjPKWhNL.png";

            let cover = fetch_image(&reqwest::Client::new(), Some(url))
                .await?
                .expect("nada");

            let mut item_step = 0;

            let (_, start_row) = cursor::position()?;

            for _ in 0..number_of_items {
                let config = viuer::Config {
                    x: if item_step == 0 { 1 } else { 0 },
                    width: Some(u32::from(cover_width)),
                    absolute_offset: false,
                    ..Default::default()
                };

                viuer::print(&cover, &config)?;

                if item_step < item_count {
                    execute!(
                        stdout,
                        cursor::MoveToRow(start_row),
                        cursor::MoveRight(cover_width + gap)
                    )?;

                    item_step = item_step + 1;
                } else {
                    execute!(stdout, cursor::MoveToColumn(0))?;

                    item_step = 0
                }
            }
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

async fn fetch_anime_search(
    client: &reqwest::Client,
    query: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<AnimeSearchResults, Box<dyn std::error::Error>> {
    let variables = anime_search::Variables {
        query,
        page,
        per_page,
    };

    let request_body = AnimeSearch::build_query(variables);

    let response: Response<anime_search::ResponseData> = client
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

    let page = response
        .data
        .ok_or_else(|| "response contained no data")?
        .page
        .ok_or_else(|| "response contained no page")?;

    let has_next_page = page
        .page_info
        .ok_or_else(|| "response contained no page info")?
        .has_next_page
        .unwrap_or_default();

    let items: Vec<Anime> = page
        .results
        .ok_or_else(|| "response contained no results")?
        .into_iter()
        .flatten()
        .map(Into::into)
        .collect();

    Ok(AnimeSearchResults {
        items,
        has_next_page,
    })
}

async fn fetch_anime_completion_search(
    client: &reqwest::Client,
    query: String,
) -> Result<Vec<AnimeCompletion>, Box<dyn std::error::Error>> {
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

    let results = results
        .into_iter()
        .map(|anime| {
            let id = anime.id;
            let title = anime
                .title
                .and_then(|title| title.romaji)
                .unwrap_or_else(|| "No Title".into());

            AnimeCompletion { id, title }
        })
        .collect();

    Ok(results)
}

async fn fetch_anime_info(
    client: &reqwest::Client,
    id: i64,
) -> Result<Anime, Box<dyn std::error::Error>> {
    let variables = anime_info::Variables { id };
    let request_body = AnimeInfo::build_query(variables);

    let response: Response<anime_info::ResponseData> = client
        .post(ANILIST_GRAPHQL_URL)
        .json(&request_body)
        .send()
        .await?
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

    let media = response
        .data
        .ok_or_else(|| Error::other("response contained no data"))?
        .media
        .ok_or_else(|| Error::other("response contained no media"))?;

    Ok(media.into())
}

fn print_wrapped_lines(
    stdout: &mut std::io::Stdout,
    text: &str,
    shift_right: u16,
    space_available: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let lines = textwrap::wrap(text, space_available);

    for line in lines {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "{line}")?;
    }

    Ok(())
}

fn print_anime_info(
    media: &Anime,
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
        const COVER_WIDTH: u32 = 27;

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

    let space_available = usize::from(
        terminal_columns
            .saturating_sub(shift_right)
            .saturating_sub(1),
    );

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
            if native == romaji {
                writeln!(stdout, "{native}")?;
                print_wrapped_lines(&mut stdout, native, shift_right, space_available)?
            } else {
                print_wrapped_lines(&mut stdout, native, shift_right, space_available)?;
                print_wrapped_lines(&mut stdout, romaji, shift_right, space_available)?
            }
        }
        (Some(romaji), None) => {
            print_wrapped_lines(&mut stdout, romaji, shift_right, space_available)?
        }
        (None, Some(native)) => {
            print_wrapped_lines(&mut stdout, native, shift_right, space_available)?
        }
        (None, None) => {
            execute!(stdout, cursor::MoveToColumn(shift_right))?;
            writeln!(stdout, "No Title")?
        }
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

    if let Some(genres) = &media.genres {
        let genres: Vec<&str> = genres.iter().map(String::as_str).collect();

        if !genres.is_empty() {
            let genres = format!("Genres: {}", genres.join(" · "));

            print_wrapped_lines(&mut stdout, &genres, shift_right, space_available)?
        };
    }

    writeln!(stdout)?;

    if let Some(description) = &media.description {
        let description: String = scraper::Html::parse_fragment(description)
            .root_element()
            .text()
            .collect();

        print_wrapped_lines(&mut stdout, &description, shift_right, space_available)?;

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

async fn print_anime_search(
    client: &reqwest::Client,
    results: AnimeSearchResults,
) -> Result<(), Box<dyn std::error::Error>> {
    let has_next_page = results.has_next_page;

    let items: Vec<(Anime, Option<DynamicImage>)> =
        futures::future::join_all(results.items.into_iter().map(|anime| async {
            let cover = fetch_image(client, anime.cover_url.as_deref())
                .await
                .ok()
                .flatten();

            (anime, cover)
        }))
        .await;

    for (anime, cover) in items {
        print_anime_info(&anime, cover.as_ref())?;

        println!("\n");
    }

    if has_next_page {
        println!("page 2 is available")
    }

    Ok(())
}
