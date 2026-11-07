const ROM_START: usize = 0x200;
const FONT_START: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - ROM_START;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: Vec<u16>,
}

impl Chip8 {
    fn new() -> Self {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(FONT.as_slice());
        let stack: Vec<u16> = Vec::new();
        Chip8 {
            memory,
            v: [0; 16],
            i: 0,
            pc: ROM_START as u16,
            stack,
        }
    }

    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.is_empty() {
            return Err("Empty ROM".to_string());
        }
        if rom.len() > MAX_ROM_SIZE {
            return Err("ROM is too large".to_string());
        }
        let mut machine = Self::new();
        machine.memory[ROM_START..ROM_START + rom.len()].copy_from_slice(rom);
        Ok(machine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_loads_font_and_sets_pc() {
        let machine = Chip8::new();
        assert_eq!(machine.pc, 0x200);
        assert_eq!(machine.memory[0x50], 0xF0);
    }

    #[test]
    fn load_valid_rom() {
        let rom: [u8; 3] = [0x80, 0x91, 0x45];
        let machine_res = Chip8::from_rom(&rom);
        assert!(machine_res.is_ok());
        let machine = machine_res.unwrap();
        assert_eq!(&machine.memory[ROM_START..ROM_START+3], &rom);
    }

    #[test]
    fn allow_max_rom() {
        let rom: [u8; MAX_ROM_SIZE] = [0x80; MAX_ROM_SIZE];
        assert!(Chip8::from_rom(&rom).is_ok());
    }

    #[test]
    fn fail_empty_rom() {
        assert!(Chip8::from_rom(&[]).is_err());
    }

    #[test]
    fn fail_too_big_rom() {
        let rom: [u8; MAX_ROM_SIZE + 1] = [0x80; MAX_ROM_SIZE + 1];
        assert!(Chip8::from_rom(&rom).is_err());
    }
}
