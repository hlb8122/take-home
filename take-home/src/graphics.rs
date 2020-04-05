use client::types::ItemMetadata;

use sdl2::{
    image::LoadTexture,
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator},
    ttf::Font,
    video::WindowContext,
};

use std::{cmp::Ordering, path::Path, time::Instant};

pub const HEADER_TEXT_HEIGHT: u32 = 24;
pub const BLURB_TEXT_HEIGHT: u32 = 20;

pub struct GfxState<'a> {
    window_width: u32,
    window_height: u32,
    item_width: u32,
    item_height: u32,
    item_padding: u32,
    texture_creator: &'a TextureCreator<WindowContext>,
    selection: usize,
    shift: i32,
    n_games: usize,
    textures: Option<Vec<Texture<'a>>>,
    item_metadata: Vec<ItemMetadata>,
}

impl<'a> GfxState<'a> {
    pub fn new(
        window_width: u32,
        window_height: u32,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Self {
        let item_padding = window_width / 40;
        let item_width = window_width / 8;
        let item_height = item_width * 9 / 16;

        GfxState {
            window_width,
            window_height,
            item_width,
            item_height,
            item_padding,
            texture_creator,
            selection: 0,
            shift: 0,
            textures: None,
            n_games: 0,
            item_metadata: Vec::with_capacity(16),
        }
    }

    pub fn reset(&mut self) {
        self.textures = None;
        self.n_games = 0;
        self.shift = 0;
        self.selection = 0;
        self.item_metadata = Vec::with_capacity(16);
    }

    /// Shift selection right
    pub fn selection_right(&mut self) {
        self.selection = (self.selection + 1) % self.n_games;

        if self.selection == 0 {
            self.shift = 0;
            return;
        }

        let selected_rectangle = self.get_item_rectangle(self.selection);
        if selected_rectangle.right() + (self.item_width / 2 + self.item_padding) as i32
            > self.window_width as i32
        {
            self.shift -= (self.item_width + self.item_padding) as i32;
        }
    }

    pub fn get_item_metadata(&self, index: usize) -> Option<&ItemMetadata> {
        self.item_metadata.get(index)
    }

    /// Shift selection right
    pub fn selection_left(&mut self) {
        if self.selection == 0 {
            self.selection = self.n_games - 1;
        } else {
            self.selection -= 1;
        }

        if self.selection == 0 {
            self.shift = 0;
            return;
        }

        if self.selection == self.n_games - 1 {
            self.shift -= (self.n_games as i32 - 6) * (self.item_width + self.item_padding) as i32;
            return;
        }

        let selected_rectangle = self.get_item_rectangle(self.selection);
        if selected_rectangle.left() < (self.item_width / 2 + self.item_padding) as i32 {
            self.shift += (self.item_width + self.item_padding) as i32;
        }
    }

    pub fn n_games(&self) -> usize {
        self.n_games
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn get_item_texture_mut(&mut self, index: usize) -> Option<&mut Texture<'a>> {
        self.textures.as_mut().unwrap().get_mut(index)
    }

    /// Initialize graphics
    pub fn init(&mut self, item_metadatas: &mut Vec<ItemMetadata>) {
        // Don't re-initialize
        if self.textures.is_none() {
            let n_games = item_metadatas.len();

            // Initialize textures
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

            // Take item_metadatas from network state
            self.item_metadata.append(item_metadatas);
        }
    }

    pub fn drain_images(&mut self, image_paths: &mut Vec<(usize, String)>) -> Result<(), String> {
        for (i, image_path) in image_paths.drain(..) {
            self.textures.as_mut().unwrap()[i] =
                self.texture_creator.load_texture(Path::new(&image_path))?;
        }
        Ok(())
    }

    // Return rectangles above and below selected item
    pub fn get_selected_rectangles(&self) -> (Rect, Rect) {
        let item_height_enlarged = self.item_height * 3 / 2;
        let y = (self.window_height / 3 - item_height_enlarged / 4) as i32;

        let y1 = y - HEADER_TEXT_HEIGHT as i32;
        let y2 = y + item_height_enlarged as i32;

        let x = self.shift
            + self.item_padding as i32
            + (self.selection as i32 * (self.item_padding + self.item_width) as i32);
        let width = self.item_width * 3 / 2;

        (
            Rect::new(x, y1, width, HEADER_TEXT_HEIGHT),
            Rect::new(x, y2, width, BLURB_TEXT_HEIGHT),
        )
    }

    pub fn get_item_rectangle(&self, game_index: usize) -> Rect {
        let y = (self.window_height / 3) as i32;
        let game_index_i32 = game_index as i32;
        match game_index.cmp(&self.selection) {
            Ordering::Less => {
                // Less than selected index
                let x = self.shift
                    + self.item_padding as i32
                    + (game_index_i32 * (self.item_padding + self.item_width) as i32);
                Rect::new(x, y, self.item_width, self.item_height)
            }
            Ordering::Equal => {
                // Selected index
                let x = self.shift
                    + self.item_padding as i32
                    + (game_index_i32 * (self.item_padding + self.item_width) as i32);
                let enlarged_item_width = self.item_width * 3 / 2;
                let enlarged_item_height = self.item_height * 3 / 2;
                Rect::new(
                    x,
                    y - (enlarged_item_height / 4) as i32,
                    enlarged_item_width,
                    enlarged_item_height,
                )
            }
            Ordering::Greater => {
                // More than selected index
                let enlarged_item_width = self.item_width as i32 * 3 / 2;
                let x = self.shift
                    + self.item_padding as i32
                    + enlarged_item_width
                    + self.item_padding as i32
                    + (game_index_i32 * (self.item_padding + self.item_width) as i32)
                    - (self.item_padding + self.item_width) as i32;
                Rect::new(x, y, self.item_width, self.item_height)
            }
        }
    }
}

pub fn get_loading_texture<'a, 'ttf>(
    font: &Font<'ttf, 'static>,
    start: Instant,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>, String> {
    let now = Instant::now();
    let millis = now.duration_since(start).as_millis() % 1500;
    if millis < 1500 / 3 {
        get_text_texture("Fetching Games.  ", font, texture_creator)
    } else if millis < 1500 * 2 / 3 {
        get_text_texture("Fetching Games.. ", font, texture_creator)
    } else {
        get_text_texture("Fetching Games...", font, texture_creator)
    }
}

pub fn get_text_texture<'a, 'ttf>(
    text: &str,
    font: &Font<'ttf, 'static>,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>, String> {
    let loading_surface = font
        .render(text)
        .blended(Color::RGBA(255, 255, 255, 255))
        .map_err(|e| e.to_string())?;
    texture_creator
        .create_texture_from_surface(&loading_surface)
        .map_err(|e| e.to_string())
}
