use chip8_core::*;
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
    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to initialize video subsystem");

    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("Failed to create window");

    let canvas = window.into_canvas().present_vsync().build().expect("Failed to create canvas");
    let event_pump = sdl_context.event_pump().expect("Failed to create event pump");

    (canvas, event_pump)
}

/// Initializes the Chip-8 emulator and loads the game.
fn initialize_chip8(game_path: &str) -> Chip8 {
    let mut chip8 = Chip8::initialize();

    let mut rom_file = File::open(game_path).unwrap_or_else(|_| {
        eprintln!("Failed to open file: {}", game_path);
        std::process::exit(1);
    });

    let mut buffer = Vec::new();
    rom_file.read_to_end(&mut buffer).expect("Failed to read ROM file");

    chip8.load_program(&buffer);
    chip8
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
            Event::Quit { .. } => std::process::exit(0),
            Event::KeyDown { keycode: Some(key), .. } => {
                if let Some(mapped_key) = map_key_to_button(key) {
                    chip8.set_key_state(mapped_key, true);
                }
            }
            Event::KeyUp { keycode: Some(key), .. } => {
                if let Some(mapped_key) = map_key_to_button(key) {
                    chip8.set_key_state(mapped_key, false);
                }
            }
            _ => {}
        }
    }
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

