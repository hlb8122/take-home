use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Cut {
    pub src: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub cuts: HashMap<String, Cut>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Mlb {
    headline: String,
    subhead: String,
    blurb: String,
    photo: Photo,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Recap {
    mlb: Mlb,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Editorial {
    recap: Recap,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    editorial: Option<Editorial>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub game_date: String,
    pub content: Content,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DateItem {
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub dates: Vec<DateItem>,
}

#[derive(Debug)]
pub struct ThumbnailData {
    pub date: String,
    pub headline: String,
    pub subhead: String,
    pub blurb: String,
    pub photos: HashMap<String, String>,
}

impl Schedule {
    /// Compactify the JSON into the relevant thumbnail data.
    ///
    /// This will filter out any games which have missing editorials.
    pub fn into_thumbnail_data(self) -> Vec<Vec<ThumbnailData>> {
        self.dates
            .into_iter()
            .map(move |item| {
                item.games
                    .into_iter()
                    .filter_map(move |game| {
                        if let Some(editorial) = game.content.editorial {
                            let mlb = editorial.recap.mlb;
                            let photos = mlb
                                .photo
                                .cuts
                                .into_iter()
                                .map(move |(res, cut)| (res, cut.src))
                                .collect();
                            Some(ThumbnailData {
                                date: game.game_date,
                                headline: mlb.headline,
                                subhead: mlb.subhead,
                                blurb: mlb.blurb,
                                photos,
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect()
    }
}
