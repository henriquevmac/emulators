const ROM_START: usize = 0x200;
const FONT_START: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - ROM_START;

const X_MASK: u16 = 0x0F00;
const Y_MASK: u16 = 0x00F0;
const N_MASK: u16 = 0x000F;
const NN_MASK: u16 = 0x00FF;
const NNN_MASK: u16 = 0x0FFF;
const KIND_MASK: u16 = 0xF000;

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

    pub fn fetch(&mut self) -> u16 {
        let first = self.memory[self.pc as usize] as u16;
        self.pc += 1;
        let second = self.memory[self.pc as usize] as u16;
        self.pc += 1;
        first << 8 | second
    }

    fn get_x(opcode: u16) -> usize {
        ((opcode & X_MASK) >> 8) as usize
    }

    fn get_y(opcode: u16) -> usize {
        ((opcode & Y_MASK) >> 4) as usize
    }

    fn get_n(opcode: u16) -> u8 {
        (opcode & N_MASK) as u8
    }

    fn get_nn(opcode: u16) -> u8 {
        (opcode & NN_MASK) as u8
    }

    fn get_nnn(opcode: u16) -> u16 {
        opcode & NNN_MASK
    }

    fn get_kind(opcode: u16) -> u8 {
        ((opcode & KIND_MASK) >> 12) as u8
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
        assert_eq!(&machine.memory[ROM_START..ROM_START + 3], &rom);
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

    #[test]
    fn fetching_correctly() {
        let mut machine = Chip8::from_rom(&[0x12, 0x34, 0xA2, 0xF0]).unwrap();
        let opcode = machine.fetch();
        assert_eq!(opcode, 0x1234);
        assert_eq!(machine.pc, 0x202);
        let opcode = machine.fetch();
        assert_eq!(opcode, 0xA2F0);
        assert_eq!(machine.pc, 0x204);
    }

    #[test]
    fn decoding_correctly() {
        let opcode: u16 = 0xD123;
        assert_eq!(Chip8::get_x(opcode), 1);
        assert_eq!(Chip8::get_y(opcode), 2);
        assert_eq!(Chip8::get_n(opcode), 3);
        assert_eq!(Chip8::get_nn(opcode), 0x23);
        assert_eq!(Chip8::get_nnn(opcode), 0x123);
        assert_eq!(Chip8::get_kind(opcode), 0xD);
    }
}
