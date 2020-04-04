extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowBuilder;
use std::env;
use std::path::Path;

const BACKGROUND_PATH: &str = "./assets/background.jpg";

pub fn main() -> Result<(), String> {
    let background_path = Path::new(BACKGROUND_PATH);
    
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Get display mode
    let display_mode = video_subsystem.desktop_display_mode(0)?;

    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Take Home", display_mode.w as u32, display_mode.h as u32)
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
