# Take Home

## Installation

### Debian/Ubuntu

```bash
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
cargo build --release
```

Executable will be located at `./target/release/take-home`.

Package the executable up with the `./assets/` folder.

### Windows

Follow the steps [here](https://github.com/Rust-SDL2/rust-sdl2#windows-with-build-script) (repeating for sdl2_image and sdl2_tff).

```bash
cargo build --release
```

Executable will be located at `./target/release/take-home.exe`.

Package the executable up with the `./take-home/*.dll` files found in the folder and the `./assets/` folder.

### MacOS

```bash
brew install sdl2 sdl2_image sdl2_ttf
cargo build --release
```

Executable will be located at `./target/release/take-home`.

Package the executable up with the `./assets/` folder.
