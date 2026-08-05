use crossterm::{cursor, execute, style::Print, terminal};
use image::DynamicImage;
use std::io::{Write, stdout};

use super::models::{Anime, Character};

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

#[derive(PartialEq, Copy, Clone)]
pub enum Style {
    Large,
    Small,
}

impl Style {
    fn cover_width(self) -> u32 {
        match self {
            Self::Large => 27,
            Self::Small => 10,
        }
    }
}

pub fn print_anime_info(
    media: &Anime,
    cover: Option<&DynamicImage>,
    style: Style,
) -> Result<(), Box<dyn std::error::Error>> {
    // Large style variant:
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
    //
    // Small style variant:
    // ┌───────┐  [97986] Made in Abyss
    // │       │  TV · 13 episodes · Summer 2017
    // │       │
    // │ cover │
    // │       │
    // │       │
    // └───────┘
    let mut stdout = stdout();
    let mut shift_right = 0;
    let mut row_below_image = 0;

    let (terminal_columns, terminal_rows) = terminal::size()?;

    if let Some(cover) = cover {
        let cover_resized = viuer::resize(cover, Some(style.cover_width()), None);

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
            width: Some(style.cover_width()),
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

    match style {
        Style::Small => {
            let id = media.id;
            let title = romaji.or(romaji).unwrap_or("No Title");

            print_wrapped_lines(
                &mut stdout,
                &format!("[{id}] {title}"),
                shift_right,
                space_available,
            )?;
        }
        Style::Large => {
            match (romaji, native) {
                (Some(romaji), Some(native)) => {
                    print_wrapped_lines(&mut stdout, romaji, shift_right, space_available)?;

                    if native != romaji {
                        print_wrapped_lines(&mut stdout, native, shift_right, space_available)?
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
        }
    }

    let mut details = Vec::new();

    if let Some(format) = &media.format {
        details.push(format.to_string());
    }

    if let Some(episodes) = media.episodes {
        let label = if episodes == 1 { "episode" } else { "episodes" };

        details.push(format!("{episodes} {label}"));
    }

    match style {
        Style::Small => {
            if let (Some(year), Some(season)) = (media.season_year, &media.season) {
                details.push(format!("{season} {year}"));
            }
        }
        Style::Large => {
            if let Some(season_year) = media.season_year {
                details.push(season_year.to_string());
            }
        }
    }

    if !details.is_empty() {
        execute!(stdout, cursor::MoveToColumn(shift_right))?;
        writeln!(stdout, "{}", details.join(" · "))?
    }

    if style == Style::Large {
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
    }

    let (_, current_row) = cursor::position()?;

    if current_row < row_below_image {
        execute!(stdout, cursor::MoveTo(0, row_below_image))?
    }

    Ok(())
}

pub fn print_character_info(
    character: &Character,
    cover_image: Option<&DynamicImage>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ┌──────────────────────┐  François Claire
    // │                      │  クレア・フランソワ
    // │                      │
    // │                      │  Age: 15-17
    // │                      │  Gender: Female
    // │                      │  Birthday: Mar 28
    // │                      │
    // │     COVER IMAGE      │  **Height:** 157 cm (5'2\")\n\nClaire François is a proud noble who
    // │                      │  in the original [Revolution] is the villainess who bullies the
    // │                      │  heroine. Claire is prideful with a competitive side to prove her
    // │                      │  worth as well as her family name. Her threatening behavior
    // │                      │  conceals her fragile heart that comes from her childhood
    // │                      │  experiences. Becomes more prone to jealousy as people she cares
    // │                      │  for may leave her. Her arrogant personality is mainly influenced
    // └──────────────────────┘  by her father doting on her in her childhood more than anything.
    //
    // Voice
    //   [122695] Nanami Karin
    //
    // Appears in
    //   [158704] Watashi no Oshi wa Akuyaku Reijou
    //   [168999] Watashi no Oshi wa Akuyaku Reijou Rae to Claire ni Ichimon Ittou

    let mut stdout = stdout();
    let mut shift_right = 0;
    let mut row_below_image = 0;

    let (terminal_columns, terminal_rows) = terminal::size()?;

    if let Some(image) = cover_image {
        let image_resized = viuer::resize(image, Some(Style::Large.cover_width()), None);

        let (cover_columns, cover_rows) = (
            u16::try_from(image_resized.width())?,
            u16::try_from(image_resized.height().div_ceil(2))?,
        );

        let (_, current_row) = cursor::position()?;

        let available_rows = terminal_rows.saturating_sub(current_row).saturating_sub(1);

        let needs_scrolling = available_rows < cover_rows;

        if needs_scrolling {
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
            width: Some(Style::Large.cover_width()),
            restore_cursor: true,
            absolute_offset: false,
            ..Default::default()
        };

        viuer::print(image, &config)?;

        row_below_image = current_row.saturating_add(cover_rows);
        shift_right = cover_columns + 3;
    }

    let space_available = usize::from(
        terminal_columns
            .saturating_sub(shift_right)
            .saturating_sub(1),
    );

    let last = character
        .name
        .as_ref()
        .and_then(|name| name.last.as_deref());

    let first = character
        .name
        .as_ref()
        .and_then(|name| name.first.as_deref());

    let native = character
        .name
        .as_ref()
        .and_then(|name| name.native.as_deref());

    match (last, first, native) {
        (Some(last), Some(first), Some(native)) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{last} {first}\n")),
            cursor::MoveToColumn(shift_right),
            Print(format!("{native}\n"))
        )?,
        (Some(last), Some(first), None) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{last} {first}\n"))
        )?,
        (Some(last), None, None) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{last}\n"))
        )?,
        (None, None, None) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print("No Name\n")
        )?,
        (None, None, Some(native)) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{native}\n"))
        )?,
        (None, Some(first), Some(native)) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{first}\n")),
            cursor::MoveToColumn(shift_right),
            Print(format!("{native}\n")),
        )?,
        (Some(last), None, Some(native)) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{last}\n")),
            cursor::MoveToColumn(shift_right),
            Print(format!("{native}\n")),
        )?,
        (None, Some(first), None) => execute!(
            stdout,
            cursor::MoveToColumn(shift_right),
            Print(format!("{first}\n"))
        )?,
    }

    execute!(
        stdout,
        Print("\n"),
        cursor::MoveToColumn(shift_right),
        Print(format!(
            "Age: {}\n",
            character.age.as_deref().unwrap_or_else(|| "Unknown")
        ))
    )?;

    execute!(
        stdout,
        cursor::MoveToColumn(shift_right),
        Print(format!(
            "Gender: {}\n",
            character.gender.as_deref().unwrap_or_else(|| "Unknown")
        ))
    )?;

    let day = character.date_of_birth.as_ref().and_then(|date| date.day);
    let month = character.date_of_birth.as_ref().and_then(|date| date.month);

    let month: Option<&str> = match month {
        Some(1) => Some("Jan"),
        Some(2) => Some("Feb"),
        Some(3) => Some("Mar"),
        Some(4) => Some("Apr"),
        Some(5) => Some("May"),
        Some(6) => Some("Jun"),
        Some(7) => Some("Jul"),
        Some(8) => Some("Aug"),
        Some(9) => Some("Sep"),
        Some(10) => Some("Oct"),
        Some(11) => Some("Nov"),
        Some(12) => Some("Dec"),
        _ => None,
    };

    let birthday = match (day, month) {
        (Some(day), Some(month)) => format!("{month} {day}"),
        _ => "Unknown".to_owned(),
    };

    execute!(
        stdout,
        cursor::MoveToColumn(shift_right),
        Print(format!("Birthday: {birthday}\n\n"))
    )?;

    if let Some(description) = character.description.as_deref() {
        // FIXME: need to use a markdown parser + fix custom anilist tags
        // nice playground id: 126156
        let link_regex = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)")?;
        let bold_regex = regex::Regex::new(r"\*\*")?;
        let bold_regex2 = regex::Regex::new(r"__")?;
        let spoiler_regex = regex::Regex::new(r"~!.*!~")?;

        let description = link_regex.replace_all(description, "$1");
        let description = bold_regex.replace_all(&description, "");
        let description = bold_regex2.replace_all(&description, "");
        let description = spoiler_regex.replace_all(&description, "");

        print_wrapped_lines(&mut stdout, &description, shift_right, space_available)?;
    }

    let (_, current_row) = cursor::position()?;

    if current_row < row_below_image {
        execute!(stdout, cursor::MoveTo(0, row_below_image))?
    }

    if !character.voice_actors.is_empty() {
        writeln!(stdout, "\nVoice")?;

        for voice_actor in character.voice_actors.iter() {
            let id = voice_actor.id;

            let last = voice_actor
                .name
                .as_ref()
                .and_then(|name| name.last.as_deref());

            let first = voice_actor
                .name
                .as_ref()
                .and_then(|name| name.first.as_deref());

            let name: String = match (last, first) {
                (Some(last), Some(first)) => format!("{last} {first}"),
                (Some(last), None) => last.to_owned(),
                (None, Some(first)) => first.to_owned(),
                (None, None) => "No Name".to_owned(),
            };

            writeln!(stdout, "  [{id}] {name}")?
        }
    }

    if !character.appears_in.is_empty() {
        writeln!(stdout, "\nAppears in")?;

        for appears_in in character.appears_in.iter() {
            let id = appears_in.id;
            let title = appears_in.title.as_deref().unwrap_or_else(|| "No Title");

            writeln!(stdout, "  [{id}] {title}")?
        }
    }

    Ok(())
}
