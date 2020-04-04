pub mod networking;

use client::{types::ThumbnailData, MlbClient};
use networking::NetworkState;

use parking_lot::RwLock;
use sdl2::{
    event::Event,
    image::{InitFlag, LoadTexture},
    keyboard::Keycode,
    rect::Rect,
    render::{RenderTarget, Texture, TextureCreator},
    video::WindowContext,
};

use std::{collections::HashMap, path::Path, sync::Arc};

const BACKGROUND_PATH: &str = "./assets/background.jpg";

const TEXTURE_WIDTH: u32 = 320;
const TEXTURE_HEIGHT: u32 = 180;
const N_TEXTURE_PIXELS: usize = (TEXTURE_HEIGHT * TEXTURE_WIDTH) as usize;

pub struct GfxState<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    selected: usize,
    textures: Option<Vec<Texture<'a>>>,
}

impl<'a> GfxState<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        GfxState {
            texture_creator,
            selected: 0,
            textures: None,
        }
    }

    /// Get reference to vector of textures.
    ///
    /// This will panic if unitialized.
    fn get_textures(&self) -> &Vec<Texture<'a>> {
        self.textures.as_ref().unwrap()
    }

    /// Get reference to vector of textures.
    ///
    /// This will panic if unitialized.
    fn get_textures_mut(&mut self) -> &mut Vec<Texture<'a>> {
        self.textures.as_mut().unwrap()
    }

    /// Initialize textures
    fn init(&mut self, n_games: usize) {
        // Don't re-initialize
        if self.textures.is_none() {
            let mut textures = Vec::with_capacity(n_games);
            for i in 0..n_games {
                let texture = self
                    .texture_creator
                    .create_texture_static(None, TEXTURE_WIDTH, TEXTURE_HEIGHT)
                    .unwrap();
                textures.push(texture);
            }
            self.textures = Some(textures);
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
    let background_texture = texture_creator.load_texture(background_path)?;

    canvas.present();

    // Initialize MLB client
    let client = MlbClient::new();

    // Initialize program state
    let network_state = Arc::new(RwLock::new(NetworkState::FetchingJson));

    let startup_task = networking::startup_procedure(client.clone(), network_state.clone());
    tokio::spawn(startup_task);

    // Initialize graphics state
    let mut gfx_state = GfxState::new(&texture_creator);

    'mainloop: loop {
        // Reset canvas
        canvas.clear();

        // Render background texture
        canvas.copy(&background_texture, None, None)?;

        match &*network_state.read() {
            NetworkState::Error => {
                // Display error page
            }
            NetworkState::FetchingJson => {
                // Displaying loading page
            }
            NetworkState::FetchingImages(thumbnails, image_map) => {
                // Initialize if required
                let n_games = thumbnails.len();
                gfx_state.init(n_games);

                // Display games
                for texture in gfx_state.get_textures_mut() {
                    texture
                        .update(None, &[0; N_TEXTURE_PIXELS], TEXTURE_WIDTH as usize)
                        .map_err(|err| err.to_string())?;
                    let rectangle = Rect::new(300, 300, TEXTURE_WIDTH, TEXTURE_HEIGHT);
                    canvas.copy(texture, None, rectangle)?;
                }
            }
            NetworkState::Done => {
                // Display games
                for texture in gfx_state.get_textures_mut() {
                    texture
                        .update(None, &[255; N_TEXTURE_PIXELS], TEXTURE_WIDTH as usize)
                        .map_err(|err| err.to_string())?;
                    let rectangle = Rect::new(300, 300, TEXTURE_WIDTH, TEXTURE_HEIGHT);
                    println!("{:?}", rectangle);
                    canvas.copy(texture, None, rectangle)?;
                }
            }
        }

        canvas.present();

        // Check events
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Key right
                    println!("pressed right");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Key left
                    println!("pressed left");
                }
                _ => {}
            }
        }
    }

    Ok(())
}
