pub mod graphics;
pub mod networking;

use client::MlbClient;
use graphics::*;
use networking::NetworkState;

use parking_lot::Mutex;
use sdl2::{
    event::Event,
    image::{InitFlag, LoadTexture},
    keyboard::Keycode,
    rect::Rect,
    ttf::Font,
};

use std::{path::Path, sync::Arc, time::Instant};

const BACKGROUND_PATH: &str = "./assets/background.jpg";
const FONT_PATH: &str = "./assets/RobotoMono-Regular.ttf";

/// Split into lines so that text may fit inside rectangles.
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

    let mut date = time::date!(2018 - 06 - 10);
    let task = networking::startup_procedure(date, client.clone(), network_state.clone());
    tokio::spawn(task);

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
                    gfx_state.drain_images(image_paths)?;
                }
                NetworkState::Done(item_metadatas, image_paths) => {
                    // Initialize if required
                    gfx_state.init(item_metadatas);
                    gfx_state.drain_images(image_paths)?;
                    networking_complete = true;
                }
            }
        }

        // Add textures
        for i in 0..gfx_state.n_games() {
            let rectangle = gfx_state.get_item_rectangle(i);
            let texture = gfx_state.get_item_texture_mut(i).unwrap(); // This is safe after initialization

            canvas.copy(texture, None, rectangle)?;

            if i == gfx_state.selection() {
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
                    if *network_state.lock() != NetworkState::FetchingJson {
                        gfx_state.selection_right();
                    }
                }
                // Key left
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if *network_state.lock() != NetworkState::FetchingJson {
                        gfx_state.selection_left();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // TODO: Remove this condition by terminating prior future early using channel
                    if networking_complete {
                        gfx_state.reset();
                        *network_state.lock() = NetworkState::FetchingJson;
                        networking_complete = false;
                        date = date.next_day();
                        let task = networking::startup_procedure(
                            date,
                            client.clone(),
                            network_state.clone(),
                        );
                        tokio::spawn(task);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // TODO: Remove this condition by terminating prior future early using channel
                    if networking_complete {
                        gfx_state.reset();
                        *network_state.lock() = NetworkState::FetchingJson;
                        networking_complete = false;
                        date = date.previous_day();
                        let task = networking::startup_procedure(
                            date,
                            client.clone(),
                            network_state.clone(),
                        );
                        tokio::spawn(task);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
