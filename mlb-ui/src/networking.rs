use client::{types::ThumbnailData, MlbClient};

use futures::prelude::*;
use parking_lot::RwLock;

use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub enum NetworkState {
    FetchingJson,
    FetchingImages(Vec<ThumbnailData>, HashMap<usize, Result<Vec<u8>, String>>),
    Error,
    Done,
}

pub async fn startup_procedure(client: MlbClient, main_state: Arc<RwLock<NetworkState>>) {
    let example_date = time::date!(2018 - 06 - 10);
    match client.get_schedule_via_date(&example_date).await {
        Err(err) => {
            // Reached error state - no thumbnail data found
            *main_state.write() = NetworkState::Error;
            return;
        }
        Ok(schedule) => {
            // Collect thumbnail data
            let mut thumbnail_data = schedule.into_thumbnail_data();
            let thumbnails = match thumbnail_data.pop() {
                Some(some) => some,
                None => {
                    // Reached error state - no thumbnail data found
                    *main_state.write() = NetworkState::Error;
                    return;
                }
            };

            // Collect image URLs
            let image_urls: Vec<Option<String>> = thumbnails
                .iter()
                .map(move |thumbnail| thumbnail.photos.get("684x385").cloned())
                .collect();

            let image_map = HashMap::with_capacity(thumbnails.len());
            *main_state.write() = NetworkState::FetchingImages(thumbnails, image_map);

            // Join all image fetching futures
            let image_fetching = future::join_all(image_urls.iter().enumerate().map(|(i, url)| {
                println!("{:?}", url);
                let client_inner = client.clone();
                let main_state_inner = main_state.clone();
                async move {
                    let image_raw = if let Some(url) = url {
                        client_inner
                            .get_image(url)
                            .await
                            .map_err(|err| err.to_string())
                    } else {
                        Err("URL not found".to_string())
                    };

                    // If in fetching images state then insert image
                    match &mut *main_state_inner.write() {
                        NetworkState::FetchingImages(_, image_map) => {
                            image_map.insert(i, image_raw);
                        }
                        _ => (),
                    }
                }
            }));
            image_fetching.await;

            *main_state.write() = NetworkState::Done;
        }
    }
}
