use chip8::Chip8;

fn run() -> Result<(), String> {
    let mut args = std::env::args();
    let filename = args
        .nth(1)
        .ok_or("Usage: chip8 <rom_file_path>".to_string())?;

    let rom_bytes =
        std::fs::read(&filename).map_err(|e| format!("Could not read {filename}: {e}"))?;

    let mut machine =
        Chip8::from_rom(&rom_bytes).map_err(|e| format!("Invalid ROM {filename}: {e}"))?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
