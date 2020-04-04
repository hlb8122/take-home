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

pub struct GfxState<'a> {
    window_width: u32,
    window_height: u32,
    item_width: u32,
    item_height: u32,
    item_padding: u32,
    texture_creator: &'a TextureCreator<WindowContext>,
    selected: i32,
    shift: i32,
    textures: Option<Vec<Texture<'a>>>,
    n_games: usize,
}

impl<'a> GfxState<'a> {
    fn new(
        window_width: u32,
        window_height: u32,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Self {
        let item_padding = window_width / 26;
        let item_width = window_width / 8;
        let item_height = item_width * 9 / 16;

        GfxState {
            window_width,
            window_height,
            item_width,
            item_height,
            item_padding,
            texture_creator,
            selected: 0,
            shift: 0,
            textures: None,
            n_games: 0,
        }
    }

    /// Shift selection right
    fn selection_right(&mut self) {
        self.selected += 1;

        if self.selected() == 0 {
            self.shift = 0;
            return;
        }

        let selected_rectangle = self.get_rectangle(self.selected());
        if selected_rectangle.right() + (self.item_width / 2 + self.item_padding) as i32
            > self.window_width as i32
        {
            self.shift -= (self.item_width + self.item_padding) as i32;
        }
    }

    /// Shift selection right
    fn selection_left(&mut self) {
        self.selected -= 1;

        if self.selected() == 0 {
            self.shift = 0;
            return;
        }

        if self.selected() == self.n_games - 1 {
            self.shift -= (self.n_games as i32 - 5) * (self.item_width + self.item_padding) as i32;
            return;
        }

        let selected_rectangle = self.get_rectangle(self.selected());
        if selected_rectangle.left() < (self.item_width / 2 + self.item_padding) as i32 {
            self.shift += (self.item_width + self.item_padding) as i32;
        }
    }

    fn n_games(&self) -> usize {
        self.n_games
    }

    fn selected(&self) -> usize {
        self.selected as usize % self.n_games
    }

    fn get_texture_mut(&mut self, index: usize) -> Option<&mut Texture<'a>> {
        self.textures.as_mut().unwrap().get_mut(index)
    }

    /// Initialize textures
    fn init(&mut self, n_games: usize) {
        // Don't re-initialize
        if self.textures.is_none() {
            let mut textures = Vec::with_capacity(n_games);
            for _ in 0..n_games {
                let texture = self
                    .texture_creator
                    .create_texture_static(None, self.item_width, self.item_height)
                    .unwrap();
                textures.push(texture);
            }
            self.textures = Some(textures);
            self.n_games = n_games;
        }
    }

    fn get_rectangle(&self, game_index: usize) -> Rect {
        let y = (self.window_height / 3) as i32;
        let game_index_i32 = game_index as i32;
        if game_index < self.selected() {
            let x = self.shift
                + self.item_padding as i32
                + (game_index_i32 * (self.item_padding + self.item_width) as i32);
            Rect::new(x, y, self.item_width, self.item_height)
        } else if game_index == self.selected() {
            let x = self.shift
                + self.item_padding as i32
                + (game_index_i32 * (self.item_padding + self.item_width) as i32);
            let enlarged_item_width = self.item_width * 3 / 2;
            let enlarged_item_height = self.item_height * 3 / 2;
            Rect::new(
                x as i32,
                (y - (enlarged_item_height / 4) as i32),
                enlarged_item_width,
                enlarged_item_height,
            )
        } else {
            let enlarged_item_width = self.item_width as i32 * 3 / 2;
            let x = self.shift
                + self.item_padding as i32
                + enlarged_item_width
                + self.item_padding as i32
                + (game_index_i32 * (self.item_padding + self.item_width) as i32)
                - (self.item_padding + self.item_width) as i32;
            Rect::new(x as i32, y as i32, self.item_width, self.item_height)
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
    let window_width = display_mode.w as u32;
    let window_height = display_mode.h as u32;

    // Initialize background and canvas
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Take Home", window_width, window_height)
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

    // Initialize MLB client
    let client = MlbClient::new();

    // Initialize program state
    let network_state = Arc::new(RwLock::new(NetworkState::FetchingJson));

    let startup_task = networking::startup_procedure(client.clone(), network_state.clone());
    tokio::spawn(startup_task);

    // Initialize graphics state
    let mut gfx_state = GfxState::new(window_width, window_height, &texture_creator);

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
                for i in 0..n_games {
                    let rectangle = gfx_state.get_rectangle(i);
                    let texture = gfx_state.get_texture_mut(i).unwrap();
                    // texture
                    //     .update(None, &[0; N_TEXTURE_PIXELS], TEXTURE_WIDTH as usize)
                    //     .map_err(|err| err.to_string())?;
                    canvas.copy(texture, None, rectangle)?;
                }
            }
            NetworkState::Done => {
                // Display games
                for i in 0..gfx_state.n_games() {
                    let rectangle = gfx_state.get_rectangle(i);
                    let texture = gfx_state.get_texture_mut(i).unwrap();
                    // texture
                    //     .update(None, &[0; N_TEXTURE_PIXELS], TEXTURE_WIDTH as usize)
                    //     .map_err(|err| err.to_string())?;
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
                    gfx_state.selection_right();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Key left
                    gfx_state.selection_left();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
