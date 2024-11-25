use chip8_core::*;
use chip8_core::{MEMORY_SIZE, STARTING_ADDRESS};
use std::env;
use std::fs::File;
use std::io::Read;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (DISPLAY_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (DISPLAY_HEIGHT as u32) * SCALE;
const CYCLES_PER_FRAME: usize = 10;

fn main() {
    // Parse command-line arguments
    let game_path = parse_arguments();

    // Initialize SDL and Chip-8 Emulator
    let (mut canvas, mut event_pump) = initialize_sdl();
    let mut chip8 = initialize_chip8(&game_path);

    // Main game loop
    run_emulator(&mut chip8, &mut canvas, &mut event_pump);
}

/// Parses the command-line arguments and returns the path to the game file.
fn parse_arguments() -> String {
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "chip8_emulator".to_string());

    args.next().unwrap_or_else(|| {
        eprintln!("Usage: {} <path_to_game>", program_name);
        std::process::exit(1);
    })
}

/// Initializes SDL2 and returns the canvas and event pump.
fn initialize_sdl() -> (Canvas<Window>, sdl2::EventPump) {
    // Step 1: Initialize SDL2
    let sdl_context = match sdl2::init() {
        Ok(context) => context,
        Err(err) => {
            eprintln!("Failed to initialize SDL2: {}", err);
            std::process::exit(1);
        }
    };

    // Step 2: Initialize video subsystem
    let video_subsystem = match sdl_context.video() {
        Ok(video) => video,
        Err(err) => {
            eprintln!("Failed to initialize SDL2 video subsystem: {}", err);
            std::process::exit(1);
        }
    };

    // Step 3: Create an SDL window
    let window = match video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
    {
        Ok(win) => win,
        Err(err) => {
            eprintln!("Failed to create SDL2 window: {}", err);
            std::process::exit(1);
        }
    };

    // Step 4: Create a canvas for rendering
    let canvas = match window.into_canvas().present_vsync().build() {
        Ok(can) => can,
        Err(err) => {
            eprintln!("Failed to create SDL2 canvas: {}", err);
            std::process::exit(1);
        }
    };

    // Step 5: Initialize event pump
    let event_pump = match sdl_context.event_pump() {
        Ok(pump) => pump,
        Err(err) => {
            eprintln!("Failed to create SDL2 event pump: {}", err);
            std::process::exit(1);
        }
    };

    // Step 6: Return the canvas and event pump
    (canvas, event_pump)
}

/// Initializes the Chip-8 emulator and loads the game.
/// Initializes the Chip-8 emulator and loads the game program.
fn initialize_chip8(game_path: &str) -> Chip8 {
    let mut chip8 = create_chip8_instance();
    let rom_data = read_game_file(game_path);
    load_rom_into_chip8(&mut chip8, &rom_data);
    chip8
}

/// Creates and returns a new Chip-8 instance.
fn create_chip8_instance() -> Chip8 {
    println!("Initializing Chip-8 emulator...");
    Chip8::initialize()
}

/// Reads the game ROM from the specified file path.
fn read_game_file(game_path: &str) -> Vec<u8> {
    println!("Reading game file from path: {}", game_path);

    let mut rom_file = File::open(game_path).unwrap_or_else(|err| {
        eprintln!("Error: Could not open file '{}'. {}", game_path, err);
        std::process::exit(1);
    });

    let mut buffer = Vec::new();
    rom_file.read_to_end(&mut buffer).unwrap_or_else(|err| {
        eprintln!("Error: Could not read file '{}'. {}", game_path, err);
        std::process::exit(1);
    });

    println!("Successfully read {} bytes from '{}'.", buffer.len(), game_path);
    buffer
}

/// Loads the ROM data into the Chip-8 emulator memory.
fn load_rom_into_chip8(chip8: &mut Chip8, rom_data: &[u8]) {
    println!("Loading ROM into Chip-8 memory...");

    if rom_data.len() > (MEMORY_SIZE - STARTING_ADDRESS as usize) {
        eprintln!(
            "Error: ROM size ({}) exceeds available memory space ({} bytes).",
            rom_data.len(),
            MEMORY_SIZE - STARTING_ADDRESS as usize
        );
        std::process::exit(1);
    }

    chip8.load_program(rom_data);
    println!("ROM successfully loaded into Chip-8 memory.");
}


/// Runs the Chip-8 emulator loop.
fn run_emulator(chip8: &mut Chip8, canvas: &mut Canvas<Window>, event_pump: &mut sdl2::EventPump) {
    'game_loop: loop {
        handle_events(chip8, event_pump);
        for _ in 0..CYCLES_PER_FRAME {
            chip8.cycle();
        }
        chip8.update_timers();
        render_display(chip8, canvas);
    }
}

/// Handles user input events.
fn handle_events(chip8: &mut Chip8, event_pump: &mut sdl2::EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => handle_quit_event(),
            Event::KeyDown { keycode: Some(key), .. } => handle_key_down_event(chip8, key),
            Event::KeyUp { keycode: Some(key), .. } => handle_key_up_event(chip8, key),
            _ => handle_other_event(event),
        }
    }
}

/// Handles the quit event by exiting the program.
fn handle_quit_event() {
    println!("Quit event received. Exiting the emulator...");
    std::process::exit(0);
}

/// Handles a key press event and updates the Chip-8 input state.
fn handle_key_down_event(chip8: &mut Chip8, key: Keycode) {
    if let Some(mapped_key) = map_key_to_button(key) {
        println!("Key pressed: {:?} -> Chip-8 button {}", key, mapped_key);
        chip8.set_key_state(mapped_key, true);
    } else {
        println!("Key pressed: {:?} (unmapped)", key);
    }
}

/// Handles a key release event and updates the Chip-8 input state.
fn handle_key_up_event(chip8: &mut Chip8, key: Keycode) {
    if let Some(mapped_key) = map_key_to_button(key) {
        println!("Key released: {:?} -> Chip-8 button {}", key, mapped_key);
        chip8.set_key_state(mapped_key, false);
    } else {
        println!("Key released: {:?} (unmapped)", key);
    }
}

/// Handles any other events not explicitly covered.
fn handle_other_event(event: Event) {
    println!("Unhandled event: {:?}", event);
}


/// Renders the Chip-8 framebuffer to the SDL canvas.
fn render_display(chip8: &Chip8, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    let framebuffer = chip8.get_framebuffer();

    for (index, &pixel) in framebuffer.iter().enumerate() {
        if pixel {
            let x = (index % DISPLAY_WIDTH) as u32;
            let y = (index / DISPLAY_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

/// Maps keyboard input to Chip-8 keys.
fn map_key_to_button(key: Keycode) -> Option<usize> {
    const KEY_MAP: [(Keycode, usize); 16] = [
        (Keycode::Num1, 0x1),
        (Keycode::Num2, 0x2),
        (Keycode::Num3, 0x3),
        (Keycode::Num4, 0xC),
        (Keycode::Q, 0x4),
        (Keycode::W, 0x5),
        (Keycode::E, 0x6),
        (Keycode::R, 0xD),
        (Keycode::A, 0x7),
        (Keycode::S, 0x8),
        (Keycode::D, 0x9),
        (Keycode::F, 0xE),
        (Keycode::Z, 0xA),
        (Keycode::X, 0x0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ];

    // Search for the key in the map and return its corresponding value
    KEY_MAP.iter().find_map(|&(k, v)| if k == key { Some(v) } else { None })
}

