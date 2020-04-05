use client::{types::ItemMetadata, MlbClient};

use futures::prelude::*;
use parking_lot::Mutex;

use std::{collections::HashMap, fs::File, path::Path, sync::Arc};

const THUMBNAIL_PATH: &str = "./assets/thumbnail/";

#[derive(Debug)]
pub enum NetworkState {
    FetchingJson,
    FetchingImages(Vec<ItemMetadata>, HashMap<usize, String>),
    Error(String),
    Done(Vec<ItemMetadata>, HashMap<usize, String>),
}

pub async fn startup_procedure(client: MlbClient, state: Arc<Mutex<NetworkState>>) {
    let example_date = time::date!(2018 - 06 - 10);
    match client.get_schedule_via_date(&example_date).await {
        Err(err) => {
            // Reached error state - no item_metadata data found
            *state.lock() = NetworkState::Error(err.to_string());
            return;
        }
        Ok(schedule) => {
            // Collect item_metadata data
            let mut item_metadata_data = schedule.into_item_metadata_data();
            let item_metadatas = match item_metadata_data.pop() {
                Some(some) => some,
                None => {
                    // Reached error state - no item_metadata data found
                    *state.lock() = NetworkState::Error("Missing item_metadata data".to_string());
                    return;
                }
            };

            // Collect image URLs
            let image_urls: Vec<(u32, Option<String>)> = item_metadatas
                .iter()
                .map(move |item_metadata| {
                    (
                        item_metadata.id,
                        item_metadata.photos.get("684x385").cloned(),
                    )
                })
                .collect();

            let image_map = HashMap::with_capacity(item_metadatas.len());
            *state.lock() = NetworkState::FetchingImages(item_metadatas, image_map);

            // Join all image fetching futures
            let image_fetching =
                future::join_all(image_urls.iter().enumerate().map(|(i, (id, url))| {
                    let client_inner = client.clone();
                    let state_inner = state.clone();
                    async move {
                        if let Some(url) = url {
                            // Game had an editorial entry
                            let raw = client_inner
                                .get_image(url)
                                .await
                                .map_err(|err| err.to_string());
                            if let Ok(raw) = raw {
                                // Image get succeeded
                                let file_path = format!("{}{}.png", THUMBNAIL_PATH, id);

                                if let Ok(()) = tokio::fs::write(&file_path, raw).await {
                                    // If in fetching images state then insert image
                                    match &mut *state_inner.lock() {
                                        NetworkState::FetchingImages(_, image_map) => {
                                            image_map.insert(i, file_path);
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        };
                    }
                }));
            image_fetching.await;

            let state = &mut *state.lock();
            match state {
                NetworkState::FetchingImages(item_metadatas, image_map) => {}
                _ => {
                    *state = NetworkState::Error("Unexpected state transition".to_string());
                }
            }
        }
    }
}
