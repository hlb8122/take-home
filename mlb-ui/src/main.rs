pub mod networking;

use client::{types::ItemMetadata, MlbClient};
use networking::NetworkState;

use parking_lot::Mutex;
use sdl2::{
    event::Event,
    image::{InitFlag, LoadTexture},
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator},
    ttf::Font,
    video::WindowContext,
};

use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};

const BACKGROUND_PATH: &str = "./assets/background.jpg";
const FONT_PATH: &str = "./assets/RobotoMono-Regular.ttf";

const HEADER_TEXT_HEIGHT: u32 = 24;
const BLURB_TEXT_HEIGHT: u32 = 20;

pub struct GfxState<'a> {
    window_width: u32,
    window_height: u32,
    item_width: u32,
    item_height: u32,
    item_padding: u32,
    texture_creator: &'a TextureCreator<WindowContext>,
    selected: usize,
    shift: i32,
    n_games: usize,
    textures: Option<Vec<Texture<'a>>>,
    item_metadata: Vec<ItemMetadata>,
}

impl<'a> GfxState<'a> {
    fn new(
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
            selected: 0,
            shift: 0,
            textures: None,
            n_games: 0,
            item_metadata: Vec::with_capacity(16),
        }
    }

    /// Shift selection right
    fn selection_right(&mut self) {
        self.selected = (self.selected + 1) % self.n_games;

        if self.selected() == 0 {
            self.shift = 0;
            return;
        }

        let selected_rectangle = self.get_item_rectangle(self.selected());
        if selected_rectangle.right() + (self.item_width / 2 + self.item_padding) as i32
            > self.window_width as i32
        {
            self.shift -= (self.item_width + self.item_padding) as i32;
        }
    }

    fn get_item_metadata(&self, index: usize) -> Option<&ItemMetadata> {
        self.item_metadata.get(index)
    }

    /// Shift selection right
    fn selection_left(&mut self) {
        if self.selected == 0 {
            self.selected = self.n_games - 1;
        } else {
            self.selected -= 1;
        }

        if self.selected() == 0 {
            self.shift = 0;
            return;
        }

        if self.selected() == self.n_games - 1 {
            self.shift -= (self.n_games as i32 - 6) * (self.item_width + self.item_padding) as i32;
            return;
        }

        let selected_rectangle = self.get_item_rectangle(self.selected());
        if selected_rectangle.left() < (self.item_width / 2 + self.item_padding) as i32 {
            self.shift += (self.item_width + self.item_padding) as i32;
        }
    }

    fn n_games(&self) -> usize {
        self.n_games
    }

    fn selected(&self) -> usize {
        self.selected
    }

    fn get_item_texture_mut(&mut self, index: usize) -> Option<&mut Texture<'a>> {
        self.textures.as_mut().unwrap().get_mut(index)
    }

    /// Initialize graphics
    fn init(&mut self, item_metadatas: &mut Vec<ItemMetadata>) {
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

    async fn drain_images(&mut self, image_paths: &mut Vec<(usize, String)>) -> Result<(), String> {
        for (i, image_path) in image_paths.drain(..) {
            self.textures.as_mut().unwrap()[i] =
                self.texture_creator.load_texture(Path::new(&image_path))?;
        }
        Ok(())
    }

    // Return rectangles above and below selected item
    fn get_selected_rectangles(&self) -> (Rect, Rect) {
        let item_height_enlarged = self.item_height * 3 / 2;
        let y = (self.window_height / 3 - item_height_enlarged / 4) as i32;

        let y1 = y - HEADER_TEXT_HEIGHT as i32;
        let y2 = y + item_height_enlarged as i32;

        let x = self.shift
            + self.item_padding as i32
            + (self.selected as i32 * (self.item_padding + self.item_width) as i32);
        let width = self.item_width * 3 / 2;

        (
            Rect::new(x, y1, width, HEADER_TEXT_HEIGHT),
            Rect::new(x, y2, width, BLURB_TEXT_HEIGHT),
        )
    }

    fn get_item_rectangle(&self, game_index: usize) -> Rect {
        let y = (self.window_height / 3) as i32;
        let game_index_i32 = game_index as i32;
        if game_index < self.selected() {
            // Less than selected index
            let x = self.shift
                + self.item_padding as i32
                + (game_index_i32 * (self.item_padding + self.item_width) as i32);
            Rect::new(x, y, self.item_width, self.item_height)
        } else if game_index == self.selected() {
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
        } else {
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

fn get_loading_texture<'a, 'ttf>(
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

fn get_text_texture<'a, 'ttf>(
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

fn new_line_splitter<'ttf>(
    text: &str,
    font: &Font<'ttf, 'static>,
    target_height: u32,
    line_width: u32,
) -> Result<Vec<String>, String> {
    let mut lines = Vec::with_capacity(3);
    let mut line = String::new();
    let mut line_len = 0;
    for word in text.split_whitespace() {
        let (width, height) = font.size_of(word).map_err(|err| err.to_string())?;
        let new_len = width * target_height / height;
        if new_len + line_len > line_width {
            lines.push(line);
            line = format!("{} ", word);
            line_len = new_len;
        } else {
            line = format!("{} {}", line, word);
            line_len += new_len;
        }
    }
    Ok(lines)
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
    let network_state = Arc::new(Mutex::new(NetworkState::FetchingJson));

    let default_date = time::date!(2018 - 06 - 10);
    let startup_task = networking::startup_procedure(default_date, client.clone(), network_state.clone());
    tokio::spawn(startup_task);

    // Initialize graphics state
    let mut gfx_state = GfxState::new(window_width, window_height, &texture_creator);
    let start_time = Instant::now();

    // Loading text rect
    let loading_height = window_height * 13 / 250;
    let loading_width = window_width / 5;
    let loading_rect = Rect::new(
        (window_width - loading_width) as i32 / 2,
        (window_height - loading_height) as i32 / 2,
        loading_width,
        loading_height,
    );

    // Load font context
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut networking_complete = false;

    'mainloop: loop {
        // Reset canvas
        canvas.clear();

        // Render background texture
        canvas.copy(&background_texture, None, None)?;

        // Drain values from networking state
        if !networking_complete {
            match &mut *network_state.lock() {
                NetworkState::Error(err) => {
                    // TODO: Display error page
                    println!("{}", err);
                    break 'mainloop;
                }
                NetworkState::FetchingJson => {
                    // Displaying loading page
                    let font =
                        ttf_context.load_font(Path::new(FONT_PATH), loading_height as u16)?;
                    let loading_texture = get_loading_texture(&font, start_time, &texture_creator)?;
                    canvas.copy(&loading_texture, None, Some(loading_rect))?;
                }
                NetworkState::FetchingImages(item_metadatas, image_paths) => {
                    // Initialize if required
                    gfx_state.init(item_metadatas);
                    gfx_state.drain_images(image_paths).await?;
                }
                NetworkState::Done(item_metadatas, image_paths) => {
                    // Initialize if required
                    gfx_state.init(item_metadatas);
                    gfx_state.drain_images(image_paths).await?;
                    networking_complete = true;
                }
            }
        }

        // Add textures
        for i in 0..gfx_state.n_games() {
            let rectangle = gfx_state.get_item_rectangle(i);
            let texture = gfx_state.get_item_texture_mut(i).unwrap(); // This is safe after initialization

            canvas.copy(texture, None, rectangle)?;

            if i == gfx_state.selected {
                // Add text
                if let Some(item_metadata) = gfx_state.get_item_metadata(i) {
                    let (header_rect, mut blurb_rect) = gfx_state.get_selected_rectangles();
                    let text_height = header_rect.height() as u16;
                    let font = ttf_context.load_font(Path::new(FONT_PATH), text_height)?;

                    // Add header
                    let header_texture =
                        get_text_texture(item_metadata.headline.as_ref(), &font, &texture_creator)?;

                    canvas.copy(&header_texture, None, Some(header_rect))?;

                    // Add blurb
                    let lines = new_line_splitter(
                        item_metadata.blurb.as_ref(),
                        &font,
                        BLURB_TEXT_HEIGHT,
                        blurb_rect.width(),
                    )?;
                    for line in lines {
                        blurb_rect.set_y(blurb_rect.y() + blurb_rect.height() as i32);

                        let blurb_texture = get_text_texture(&line, &font, &texture_creator)?;
                        canvas.copy(&blurb_texture, None, Some(blurb_rect))?;
                    }
                }
            }
        }

        // Triger render
        canvas.present();

        // Check events
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                // Escape
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                // Key right
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    gfx_state.selection_right();
                }
                // Key left
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    gfx_state.selection_left();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
