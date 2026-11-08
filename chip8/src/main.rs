use chip8::{Chip8, PIXEL_HEIGHT, PIXEL_WIDTH};
use minifb::{Key, Window, WindowOptions};

const STEPS_PER_FRAME: usize = 10;

const KEYMAP: [(Key, usize); 16] = [
    (Key::Key1, 0x1),
    (Key::Key2, 0x2),
    (Key::Key3, 0x3),
    (Key::Key4, 0xC),
    (Key::Q, 0x4),
    (Key::W, 0x5),
    (Key::E, 0x6),
    (Key::R, 0xD),
    (Key::A, 0x7),
    (Key::S, 0x8),
    (Key::D, 0x9),
    (Key::F, 0xE),
    (Key::Z, 0xA),
    (Key::X, 0x0),
    (Key::C, 0xB),
    (Key::V, 0xF),
];

fn run() -> Result<(), String> {
    let mut args = std::env::args();
    let filename = args
        .nth(1)
        .ok_or("Usage: chip8 <rom_file_path>".to_string())?;

    let rom_bytes =
        std::fs::read(&filename).map_err(|e| format!("Could not read {filename}: {e}"))?;

    let mut machine =
        Chip8::from_rom(&rom_bytes).map_err(|e| format!("Invalid ROM {filename}: {e}"))?;

    let mut window = Window::new(
        "chip8 emulator",
        PIXEL_WIDTH,
        PIXEL_HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X16,
            ..Default::default()
        },
    )
    .map_err(|e| format!("Error opening screen: {e}"))?;

    window.set_target_fps(60);

    let mut buffer: Vec<u32> = vec![0; PIXEL_HEIGHT * PIXEL_WIDTH];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (key, chip8_key) in KEYMAP {
            machine.set_key(chip8_key, window.is_key_down(key));
        }
        for _ in 0..STEPS_PER_FRAME {
            machine.step()?;
        }

        for (i, pixel) in machine.display().iter().enumerate() {
            buffer[i] = if *pixel { 0xFFFFFF } else { 0x000000 }
        }

        window
            .update_with_buffer(buffer.as_slice(), PIXEL_WIDTH, PIXEL_HEIGHT)
            .map_err(|e| format!("Error drawing to screen {e}"))?;

        machine.tick_timers();
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
