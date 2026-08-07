use std::{
    collections::HashSet,
    fmt::{self, Display},
};

use super::anilist::{anime_info, anime_search, character_info};

pub struct MediaTitle {
    pub romaji: Option<String>,
    pub native: Option<String>,
}

impl From<anime_info::AnimeInfoMediaTitle> for MediaTitle {
    fn from(value: anime_info::AnimeInfoMediaTitle) -> Self {
        Self {
            romaji: value.romaji,
            native: value.native,
        }
    }
}

impl From<anime_search::AnimeSearchPageResultsTitle> for MediaTitle {
    fn from(value: anime_search::AnimeSearchPageResultsTitle) -> Self {
        Self {
            romaji: value.romaji,
            native: value.native,
        }
    }
}

pub struct MediaCoverImage {
    pub medium: Option<String>,
    pub extra_large: Option<String>,
}

impl From<anime_info::AnimeInfoMediaCoverImage> for MediaCoverImage {
    fn from(value: anime_info::AnimeInfoMediaCoverImage) -> Self {
        Self {
            medium: None,
            extra_large: value.extra_large,
        }
    }
}

impl From<anime_search::AnimeSearchPageResultsCoverImage> for MediaCoverImage {
    fn from(value: anime_search::AnimeSearchPageResultsCoverImage) -> Self {
        Self {
            medium: value.medium,
            extra_large: None,
        }
    }
}

pub struct Anime {
    pub id: i64,
    pub title: Option<MediaTitle>,
    pub genres: Option<Vec<String>>,
    pub format: Option<MediaFormat>,
    pub status: Option<MediaStatus>,
    pub season: Option<MediaSeason>,
    pub episodes: Option<i64>,
    pub site_url: Option<String>,
    pub cover_image: Option<MediaCoverImage>,
    pub season_year: Option<i64>,
    pub description: Option<String>,
    pub average_score: Option<i64>,
}

impl From<anime_info::AnimeInfoMedia> for Anime {
    fn from(anime: anime_info::AnimeInfoMedia) -> Self {
        Self {
            id: anime.id,
            title: anime.title.map(MediaTitle::from),
            status: anime.status.map(MediaStatus::from),
            season: anime.season.map(MediaSeason::from),
            format: anime.format.map(MediaFormat::from),
            genres: anime.genres.into_iter().flatten().collect(),
            site_url: anime.site_url,
            episodes: anime.episodes,
            season_year: anime.season_year,
            description: anime.description,
            cover_image: anime.cover_image.map(MediaCoverImage::from),
            average_score: anime.average_score,
        }
    }
}

impl From<anime_search::AnimeSearchPageResults> for Anime {
    fn from(anime: anime_search::AnimeSearchPageResults) -> Self {
        Self {
            id: anime.id,
            genres: None,
            status: None,
            site_url: None,
            description: None,
            average_score: None,
            episodes: anime.episodes,
            season_year: anime.season_year,
            title: anime.title.map(|title| title.into()),
            season: anime.season.map(|season| season.into()),
            format: anime.format.map(|format| format.into()),
            cover_image: anime.cover_image.map(MediaCoverImage::from),
        }
    }
}

pub enum MediaSeason {
    FALL,
    WINTER,
    SPRING,
    SUMMER,
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

pub enum MediaFormat {
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

pub enum MediaStatus {
    HIATUS,
    FINISHED,
    RELEASING,
    CANCELLED,
    NotYetReleased,
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

pub struct Name {
    pub last: Option<String>,
    pub first: Option<String>,
    pub native: Option<String>,
}

impl From<character_info::CharacterInfoCharacterName> for Name {
    fn from(value: character_info::CharacterInfoCharacterName) -> Self {
        Self {
            last: value.last,
            first: value.first,
            native: value.native,
        }
    }
}

impl From<character_info::CharacterInfoCharacterMediaEdgesVoiceActorRolesVoiceActorName> for Name {
    fn from(
        value: character_info::CharacterInfoCharacterMediaEdgesVoiceActorRolesVoiceActorName,
    ) -> Self {
        Self {
            last: value.last,
            first: value.first,
            native: None,
        }
    }
}

pub struct FuzzyDate {
    pub day: Option<i64>,
    pub year: Option<i64>,
    pub month: Option<i64>,
}

impl From<character_info::CharacterInfoCharacterDateOfBirth> for FuzzyDate {
    fn from(value: character_info::CharacterInfoCharacterDateOfBirth) -> Self {
        Self {
            day: value.day,
            year: value.year,
            month: value.month,
        }
    }
}

pub struct CharacterImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

impl From<character_info::CharacterInfoCharacterImage> for CharacterImage {
    fn from(value: character_info::CharacterInfoCharacterImage) -> Self {
        Self {
            large: value.large,
            medium: value.medium,
        }
    }
}

pub struct CharacterAppearsIn {
    pub id: i64,
    pub title: Option<String>,
}

impl From<character_info::CharacterInfoCharacterMediaEdgesNode> for CharacterAppearsIn {
    fn from(value: character_info::CharacterInfoCharacterMediaEdgesNode) -> Self {
        Self {
            id: value.id,
            title: value.title.map(|title| title.romaji).flatten(),
        }
    }
}

pub struct CharacterVoiceActor {
    pub id: i64,
    pub name: Option<Name>,
}

impl From<character_info::CharacterInfoCharacterMediaEdgesVoiceActorRolesVoiceActor>
    for CharacterVoiceActor
{
    fn from(
        value: character_info::CharacterInfoCharacterMediaEdgesVoiceActorRolesVoiceActor,
    ) -> Self {
        Self {
            id: value.id,
            name: value.name.map(|name| name.into()),
        }
    }
}

pub struct Character {
    pub id: i64,
    pub age: Option<String>,
    pub name: Option<Name>,
    pub image: Option<CharacterImage>,
    pub gender: Option<String>,
    pub appears_in: Vec<CharacterAppearsIn>,
    pub description: Option<String>,
    pub voice_actors: Vec<CharacterVoiceActor>,
    pub date_of_birth: Option<FuzzyDate>,
}

impl Clone for character_info::CharacterInfoCharacterMediaEdgesNode {
    fn clone(&self) -> Self {
        let romaji = self.title.as_ref().and_then(|title| title.romaji.clone());

        Self {
            id: self.id.clone(),
            title: Some(character_info::CharacterInfoCharacterMediaEdgesNodeTitle { romaji }),
        }
    }
}

impl From<character_info::CharacterInfoCharacter> for Character {
    fn from(character: character_info::CharacterInfoCharacter) -> Self {
        let mut existing_voice_actor_ids = HashSet::new();

        let (voice_actors, appears_in): (Vec<_>, Vec<_>) = character
            .media
            .into_iter()
            .filter_map(|media| media.edges)
            .flatten()
            .flatten()
            .fold(
                (Vec::new(), Vec::new()),
                |(mut voice_actors, mut appears_in), edge| {
                    if let Some(node) = edge.node {
                        appears_in.push(node.into());
                    }

                    voice_actors.extend(
                        edge.voice_actor_roles
                            .into_iter()
                            .flatten()
                            .flatten()
                            .filter_map(|role| {
                                let voice_actor = role.voice_actor?;

                                existing_voice_actor_ids
                                    .insert(voice_actor.id)
                                    .then(|| voice_actor.into())
                            }),
                    );

                    (voice_actors, appears_in)
                },
            );

        Self {
            appears_in,
            voice_actors,
            id: character.id,
            age: character.age,
            name: character.name.map(Name::from),
            image: character.image.map(CharacterImage::from),
            gender: character.gender,
            description: character.description,
            date_of_birth: character.date_of_birth.map(FuzzyDate::from),
        }
    }
}
