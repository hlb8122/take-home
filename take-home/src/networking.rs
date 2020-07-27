use client::{types::ItemMetadata, MlbClient};

use futures::prelude::*;
use parking_lot::Mutex;
use time::Date;

use std::{fs, path::Path, sync::Arc};

const THUMBNAIL_PATH: &str = "./assets/thumbnails/";

#[derive(Debug, PartialEq)]
pub enum NetworkState {
    FetchingJson,
    FetchingImages(Vec<ItemMetadata>, Vec<(usize, String)>),
    Error(String),
    Done(Vec<ItemMetadata>, Vec<(usize, String)>),
}

pub async fn startup_procedure(date: Date, client: MlbClient, state: Arc<Mutex<NetworkState>>) {
    // Create thumbnail path if missing
    if !Path::new(THUMBNAIL_PATH).exists() {
        fs::create_dir_all(THUMBNAIL_PATH).unwrap(); // Unrecoverable
    }

    match client.get_schedule_via_date(&date).await {
        Err(err) => {
            // Reached error state - no item_metadata data found
            *state.lock() = NetworkState::Error(err.to_string());
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

            let image_paths = Vec::with_capacity(item_metadatas.len());
            *state.lock() = NetworkState::FetchingImages(item_metadatas, image_paths);

            // Join all image fetching futures
            let image_fetching =
                future::join_all(image_urls.iter().enumerate().map(|(i, (id, url))| {
                    let client_inner = client.clone();
                    let state_inner = state.clone();
                    async move {
                        // TODO: Check for cached image
                        if let Some(url) = url {
                            // Game had an editorial entry
                            let raw = client_inner
                                .get_image(url)
                                .await
                                .map_err(|err| err.to_string());
                            if let Ok(raw) = raw {
                                // Image received successfully
                                let file_path = format!("{}{}.png", THUMBNAIL_PATH, id);

                                if let Ok(()) = tokio::fs::write(&file_path, raw).await {
                                    // If in fetching images state then insert image
                                    if let NetworkState::FetchingImages(_, image_paths) =
                                        &mut *state_inner.lock()
                                    {
                                        image_paths.push((i, file_path));
                                    }
                                }
                            }
                        };
                    }
                }));
            image_fetching.await;

            // TODO: Speed this up
            let state_lock = &mut *state.lock();
            if let NetworkState::FetchingImages(item_metadata, image_paths) = state_lock {
                let mut new_meta = Vec::new();
                new_meta.append(item_metadata);
                let mut new_paths = Vec::new();
                new_paths.append(image_paths);
                *state_lock = NetworkState::Done(new_meta, new_paths);
            } else {
                *state_lock = NetworkState::Error("unexpected state transition".to_string());
            };
        }
    }
}
