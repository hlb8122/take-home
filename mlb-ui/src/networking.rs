use client::{types::ItemMetadata, MlbClient};

use futures::prelude::*;
use parking_lot::Mutex;

use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub enum NetworkState {
    FetchingJson,
    FetchingImages(Vec<ItemMetadata>, HashMap<usize, Result<Vec<u8>, String>>),
    Error(String),
    Done(Vec<ItemMetadata>, HashMap<usize, Result<Vec<u8>, String>>),
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
            let image_urls: Vec<Option<String>> = item_metadatas
                .iter()
                .map(move |item_metadata| item_metadata.photos.get("684x385").cloned())
                .collect();

            let image_map = HashMap::with_capacity(item_metadatas.len());
            *state.lock() = NetworkState::FetchingImages(item_metadatas, image_map);

            // Join all image fetching futures
            let image_fetching = future::join_all(image_urls.iter().enumerate().map(|(i, url)| {
                let client_inner = client.clone();
                let state_inner = state.clone();
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
                    match &mut *state_inner.lock() {
                        NetworkState::FetchingImages(_, image_map) => {
                            image_map.insert(i, image_raw);
                        }
                        _ => (),
                    }
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
