const ROM_START: usize = 0x200;
const FONT_START: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - ROM_START;
pub const PIXEL_WIDTH: usize = 64;
pub const PIXEL_HEIGHT: usize = 32;
const NUM_PIXELS: usize = PIXEL_WIDTH * PIXEL_HEIGHT;
const STACK_SIZE: usize = 16;

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
    framebuffer: [bool; NUM_PIXELS],
    keys: [bool; 16],
    sound_timer: u8,
    delay_timer: u8,
    waiting_key: Option<usize>,
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
            framebuffer: [false; NUM_PIXELS],
            keys: [false; 16],
            sound_timer: 0,
            delay_timer: 0,
            waiting_key: None,
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

    fn fetch(&mut self) -> u16 {
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

    fn get_n(opcode: u16) -> usize {
        (opcode & N_MASK) as usize
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

    fn get_pixel_idx(x: usize, y: usize) -> usize {
        y * PIXEL_WIDTH + x
    }

    pub fn step(&mut self) -> Result<(), String> {
        let opcode = self.fetch();
        let x = Self::get_x(opcode);
        let y = Self::get_y(opcode);
        let n = Self::get_n(opcode);
        let nn = Self::get_nn(opcode);
        let nnn = Self::get_nnn(opcode);
        let kind = Self::get_kind(opcode);
        match kind {
            0x0 => match nn {
                0xE0 => self.framebuffer.fill(false),
                0xEE => {
                    self.pc = self
                        .stack
                        .pop()
                        .ok_or_else(|| format!("00EE at {:#05X}: stack empty", self.pc - 2))?;
                }
                _ => unimplemented!("{opcode:#06X}"),
            },
            0x1 => self.pc = nnn,
            0x2 => {
                if self.stack.len() == STACK_SIZE {
                    return Err(format!("Stack is already full: {:#05X}", self.pc - 2));
                }
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            0x3 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6 => self.v[x] = nn,
            0x7 => self.v[x] = self.v[x].wrapping_add(nn),
            0x8 => match n {
                0x0 => self.v[x] = self.v[y],
                0x1 => {
                    self.v[x] |= self.v[y];
                    self.v[0xF] = 0;
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.v[0xF] = 0;
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.v[0xF] = 0;
                }
                0x4 => {
                    let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = carry as u8;
                }
                0x5 => {
                    let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = !borrow as u8;
                }
                0x6 => {
                    let value = self.v[y];
                    self.v[x] = value >> 1;
                    self.v[0xF] = value & 1;
                }
                0x7 => {
                    let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xF] = !borrow as u8;
                }
                0xE => {
                    let value = self.v[y];
                    self.v[x] = value << 1;
                    self.v[0xF] = value >> 7;
                }
                _ => unimplemented!("{opcode:#06X}"),
            },
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => self.i = nnn,
            0xB => self.pc = self.v[0] as u16 + nnn,
            0xC => self.v[x] = rand::random::<u8>() & nn,
            0xD => {
                let coord_x = self.v[x] as usize % PIXEL_WIDTH;
                let coord_y = self.v[y] as usize % PIXEL_HEIGHT;
                self.v[0xF] = 0;
                'rows: for row in 0..n {
                    if coord_y + row >= PIXEL_HEIGHT {
                        break;
                    }
                    let sprite = self.memory[self.i as usize + row];
                    for b in 0..8 {
                        if coord_x + b >= PIXEL_WIDTH {
                            continue 'rows;
                        }
                        let idx = Chip8::get_pixel_idx(coord_x + b, coord_y + row);
                        let sprite_bit = ((sprite >> (7 - b)) & 1) == 1;
                        if sprite_bit {
                            if self.framebuffer[idx] {
                                self.v[0xF] = 1;
                            }
                            self.framebuffer[idx] = !self.framebuffer[idx];
                        }
                    }
                }
            }
            0xE => match nn {
                0x9E => {
                    if self.keys[(self.v[x] & 0xF) as usize] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.keys[(self.v[x] & 0xF) as usize] {
                        self.pc += 2;
                    }
                }
                _ => unimplemented!("{opcode:#06X}"),
            },
            0xF => match nn {
                0x07 => self.v[x] = self.delay_timer,
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x0A => match self.waiting_key {
                    None => {
                        if let Some(key) = self.keys.iter().position(|&d| d) {
                            self.waiting_key = Some(key);
                        }
                        self.pc -= 2;
                    }
                    Some(waiting_key) => {
                        if !self.keys[waiting_key] {
                            self.v[x] = waiting_key as u8;
                            self.waiting_key = None;
                        } else {
                            self.pc -= 2;
                        }
                    }
                },
                0x29 => self.i = FONT_START as u16 + (self.v[x] as u16 & 0xF) * 5,
                0x33 => {
                    let hundreds = self.v[x] / 100;
                    let tens = self.v[x] / 10 % 10;
                    let ones = self.v[x] % 10;
                    self.memory[self.i as usize] = hundreds;
                    self.memory[self.i as usize + 1] = tens;
                    self.memory[self.i as usize + 2] = ones;
                }
                0x55 => {
                    for idx in 0..=x {
                        self.memory[self.i as usize] = self.v[idx];
                        self.i += 1;
                    }
                }
                0x65 => {
                    for idx in 0..=x {
                        self.v[idx] = self.memory[self.i as usize];
                        self.i += 1;
                    }
                }
                _ => unimplemented!("{opcode:#06X}"),
            },
            _ => unreachable!(),
        };

        Ok(())
    }

    pub fn display(&self) -> &[bool] {
        &self.framebuffer
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

    pub fn sound_active(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn tick_timers(&mut self) {
        self.sound_timer = self.sound_timer.saturating_sub(1);
        self.delay_timer = self.delay_timer.saturating_sub(1);
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

    #[test]
    fn jumping() {
        let mut machine = Chip8::from_rom(&[0x13, 0x00]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.pc, 0x300);
    }

    #[test]
    fn set_register() {
        let mut machine = Chip8::from_rom(&[0x6A, 0x42]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.v[0xA], 0x42);
        assert_eq!(machine.pc, 0x202);
    }

    #[test]
    fn add_register_no_wrap() {
        let mut machine = Chip8::from_rom(&[0x70, 0x01]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.v[0x0], 0x01);
        assert_eq!(machine.pc, 0x202);
    }

    #[test]
    fn add_register_wrap() {
        let mut machine = Chip8::from_rom(&[0x60, 0xFF, 0x70, 0x01]).unwrap();
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.v[0x0], 0x00);
        assert_eq!(machine.pc, 0x204);
    }

    #[test]
    fn register_i() {
        let mut machine = Chip8::from_rom(&[0xA1, 0x23]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.i, 0x123);
    }

    #[test]
    fn clear_screen() {
        let mut machine = Chip8::from_rom(&[0xA0, 0x50, 0xD0, 0x05, 0x00, 0xE0]).unwrap();
        machine.step().unwrap(); // I = font 0
        machine.step().unwrap(); // draw
        assert!(machine.framebuffer.iter().filter(|&&p| p).count() > 0);
        machine.step().unwrap(); // clear
        assert_eq!(machine.framebuffer.iter().filter(|&&p| p).count(), 0);
    }

    /// Loads `rom` and executes `steps` instructions.
    fn run(rom: &[u8], steps: usize) -> Chip8 {
        let mut machine = Chip8::from_rom(rom).unwrap();
        for _ in 0..steps {
            machine.step().unwrap();
        }
        machine
    }

    fn pixel(machine: &Chip8, x: usize, y: usize) -> bool {
        machine.framebuffer[Chip8::get_pixel_idx(x, y)]
    }

    fn pixels_on(machine: &Chip8) -> usize {
        machine.framebuffer.iter().filter(|&&p| p).count()
    }

    // Font "0" is F0 90 90 90 F0:
    //   ####
    //   #..#
    //   #..#
    //   #..#
    //   ####
    // which is 14 pixels.

    #[test]
    fn draw_sprite() {
        // I = font 0; draw at (V0, V0) = (0, 0), 5 rows
        let machine = run(&[0xA0, 0x50, 0xD0, 0x05], 2);
        assert_eq!(pixels_on(&machine), 14);
        assert!(pixel(&machine, 0, 0)); // top-left corner
        assert!(pixel(&machine, 3, 0)); // top-right corner
        assert!(!pixel(&machine, 1, 1)); // hollow middle
        assert!(!pixel(&machine, 4, 0)); // high bits only: column 4 stays off
        assert_eq!(machine.v[0xF], 0); // no collision
    }

    #[test]
    fn draw_twice_erases_and_sets_collision() {
        let machine = run(&[0xA0, 0x50, 0xD0, 0x05, 0xD0, 0x05], 3);
        assert_eq!(pixels_on(&machine), 0);
        assert_eq!(machine.v[0xF], 1);
    }

    #[test]
    fn draw_clips_at_right_edge() {
        // V0 = 62, V1 = 0; draw at (V0, V1)
        let machine = run(&[0xA0, 0x50, 0x60, 62, 0x61, 0, 0xD0, 0x15], 4);
        assert!(pixel(&machine, 62, 0));
        assert!(pixel(&machine, 63, 0));
        // columns 64-65 of the sprite must not wrap onto the left side
        assert!(!pixel(&machine, 0, 0));
        assert!(!pixel(&machine, 1, 0));
        // or spill into the next row
        assert!(!pixel(&machine, 0, 1));
    }

    #[test]
    fn draw_clips_at_bottom_edge() {
        // V0 = 0, V1 = 30; only rows 30 and 31 fit
        let machine = run(&[0xA0, 0x50, 0x60, 0, 0x61, 30, 0xD0, 0x15], 4);
        assert!(pixel(&machine, 0, 30));
        assert!(pixel(&machine, 0, 31));
        // rows 32-34 of the sprite must not wrap onto the top
        assert!(!pixel(&machine, 0, 0));
        assert!(!pixel(&machine, 0, 1));
        assert_eq!(pixels_on(&machine), 4 + 2); // "####" + "#..#"
    }

    #[test]
    fn draw_start_position_wraps() {
        // V0 = 66, V1 = 33 wrap to (2, 1) before drawing
        let machine = run(&[0xA0, 0x50, 0x60, 66, 0x61, 33, 0xD0, 0x15], 4);
        assert!(pixel(&machine, 2, 1));
        assert_eq!(pixels_on(&machine), 14);
    }
}

#[cfg(test)]
mod c3_tests;

#[cfg(test)]
mod c3_io_tests;
