use graphql_client::{GraphQLQuery, Response};
use image::DynamicImage;
use std::{fmt, fmt::Display, io::Error};

use super::models::{Anime, Character};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_info.graphql",
    response_derives = "Debug"
)]
pub struct AnimeInfo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/character_info.graphql",
    response_derives = "Debug"
)]
pub struct CharacterInfo;

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
pub struct AnimeSearch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/anime_characters.graphql",
    response_derives = "Debug"
)]
pub struct AnimeCharacters;

const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co/";

pub async fn fetch_image(
    client: &reqwest::Client,
    url: Option<&str>,
) -> Result<Option<DynamicImage>, Box<dyn std::error::Error>> {
    let Some(url) = url else { return Ok(None) };

    let bytes = client.get(url).send().await?.bytes().await?;

    let image = image::load_from_memory(&bytes)?;

    Ok(Some(image))
}

pub struct AnimeSearchResults {
    pub has_next_page: bool,
    pub items: Vec<Anime>,
}

pub async fn fetch_anime_search(
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

pub struct AnimeCompletion {
    pub id: i64,
    pub title: String,
}

impl Display for AnimeCompletion {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}\t{}", self.id, self.title)
    }
}

pub async fn fetch_anime_completion_search(
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

pub async fn fetch_anime_info(
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

pub async fn fetch_character_info(
    client: &reqwest::Client,
    id: i64,
) -> Result<Character, Box<dyn std::error::Error>> {
    let variables = character_info::Variables { id };
    let request_body = CharacterInfo::build_query(variables);

    let response: Response<character_info::ResponseData> = client
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

    let character = response
        .data
        .ok_or_else(|| Error::other("response contained no data"))?
        .character
        .ok_or_else(|| Error::other("response contained no character"))?;

    Ok(character.into())
}

pub struct AnimeCharactersResults {
    pub has_next_page: bool,
    pub items: Vec<Character>,
}

pub async fn fetch_anime_characters(
    client: &reqwest::Client,
    anime_id: i64,
    page: i64,
    per_page: i64,
) -> Result<AnimeCharactersResults, Box<dyn std::error::Error>> {
    let variables = anime_characters::Variables {
        page,
        anime_id,
        per_page,
    };

    let request_body = AnimeCharacters::build_query(variables);

    let response: Response<anime_characters::ResponseData> = client
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

    let page = response
        .data
        .ok_or_else(|| "response contained no data")?
        .media
        .ok_or_else(|| "response contained no media")?
        .characters
        .ok_or_else(|| "response contained no characters")?;

    let has_next_page = page
        .page_info
        .ok_or_else(|| "response contained no page info")?
        .has_next_page
        .unwrap_or_default();

    let items = page
        .edges
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|edge| Some((edge.node?, edge.voice_actor_roles).into()))
        .collect();

    Ok(AnimeCharactersResults {
        items,
        has_next_page,
    })
}
