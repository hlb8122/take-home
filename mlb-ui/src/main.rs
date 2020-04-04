extern crate sdl2;

use client::{types::ThumbnailData, MlbClient};

use parking_lot::RwLock;
use sdl2::{
    event::Event,
    image::{InitFlag, LoadTexture},
    keyboard::Keycode,
};

use std::{collections::HashMap, path::Path, sync::Arc};

const BACKGROUND_PATH: &str = "./assets/background.jpg";

pub enum MainState {
    FetchingJson,
    FetchingImages(Vec<ThumbnailData>, HashMap<usize, Vec<u8>>),
    Error,
}

async fn startup_procedure(client: MlbClient, main_state: Arc<RwLock<MainState>>) {
    let example_date = time::date!(2018 - 06 - 10);
    match client.get_schedule_via_date(&example_date).await {
        Err(err) => {
            *main_state.write() = MainState::Error;
            return;
        }
        Ok(schedule) => {
            let mut thumbnail_data = schedule.into_thumbnail_data();
            let thumbnails = match thumbnail_data.pop() {
                Some(some) => some,
                None => {
                    *main_state.write() = MainState::Error;
                    return;
                }
            };
            let image_map = HashMap::with_capacity(thumbnails.len());
            *main_state.write() = MainState::FetchingImages(thumbnails, image_map);
            println!("{:?}", thumbnail_data);
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let background_path = Path::new(BACKGROUND_PATH);

    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Get display mode
    let display_mode = video_subsystem.desktop_display_mode(0)?;
    let display_width = display_mode.w as u32;
    let display_height = display_mode.h as u32;

    // Initialize background and canvas
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Take Home", display_width, display_height)
        .fullscreen()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(background_path)?;

    canvas.copy(&texture, None, None)?;
    canvas.present();

    // Initialize MLB client
    let client = MlbClient::new();

    // Initialize program state
    let main_state = Arc::new(RwLock::new(MainState::FetchingJson));

    let startup_task = startup_procedure(client.clone(), main_state.clone());
    tokio::spawn(startup_task);

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }
    }

    Ok(())
}
