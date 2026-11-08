//! Tests for milestone C3: timers and keypad.
//!
//! Not compiled yet. These need three public methods on `Chip8`:
//!
//! ```ignore
//! pub fn tick_timers(&mut self)                     // called 60 times a second by main
//! pub fn sound_active(&self) -> bool                // true while the sound timer > 0
//! pub fn set_key(&mut self, key: usize, pressed: bool)  // key 0x0..=0xF
//! ```
//!
//! Once they exist (even as empty stubs), enable this file by adding to lib.rs,
//! next to `mod c3_tests;`:
//!
//! ```ignore
//! #[cfg(test)]
//! mod c3_io_tests;
//! ```

use super::*;

fn load(rom: &[u8]) -> Chip8 {
    Chip8::from_rom(rom).unwrap()
}

// ---------------------------------------------------------------------------
// Timers: FX15 / FX07 (delay), FX18 (sound), tick_timers()
// ---------------------------------------------------------------------------

#[test]
fn delay_timer_counts_down_once_per_tick() {
    // V0 = 5; delay = V0; (3 ticks) V1 = delay
    let mut machine = load(&[0x60, 0x05, 0xF0, 0x15, 0xF1, 0x07]);
    machine.step().unwrap();
    machine.step().unwrap();
    for _ in 0..3 {
        machine.tick_timers();
    }
    machine.step().unwrap();
    assert_eq!(machine.v[1], 2);
}

#[test]
fn delay_timer_stops_at_zero() {
    let mut machine = load(&[0x60, 0x01, 0xF0, 0x15, 0xF1, 0x07]);
    machine.step().unwrap();
    machine.step().unwrap();
    for _ in 0..5 {
        machine.tick_timers(); // must not wrap to 255
    }
    machine.step().unwrap();
    assert_eq!(machine.v[1], 0);
}

#[test]
fn delay_timer_does_not_tick_on_its_own() {
    // Timers are driven by tick_timers() (60 Hz), not by instructions.
    let mut machine = load(&[0x60, 0x05, 0xF0, 0x15, 0x70, 0x00, 0x70, 0x00, 0xF1, 0x07]);
    for _ in 0..5 {
        machine.step().unwrap();
    }
    assert_eq!(machine.v[1], 5);
}

#[test]
fn sound_plays_while_sound_timer_is_positive() {
    // V0 = 2; sound = V0
    let mut machine = load(&[0x60, 0x02, 0xF0, 0x18]);
    assert!(!machine.sound_active());
    machine.step().unwrap();
    machine.step().unwrap();
    assert!(machine.sound_active());
    machine.tick_timers();
    assert!(machine.sound_active());
    machine.tick_timers();
    assert!(!machine.sound_active());
}

// ---------------------------------------------------------------------------
// Keypad: EX9E, EXA1, FX0A
// ---------------------------------------------------------------------------

#[test]
fn skip_if_key_pressed() {
    // V0 = 5; skip if key V0 is pressed
    let mut pressed = load(&[0x60, 0x05, 0xE0, 0x9E]);
    pressed.set_key(5, true);
    pressed.step().unwrap();
    pressed.step().unwrap();
    assert_eq!(pressed.pc, 0x206);

    let mut other_key = load(&[0x60, 0x05, 0xE0, 0x9E]);
    other_key.set_key(6, true);
    other_key.step().unwrap();
    other_key.step().unwrap();
    assert_eq!(other_key.pc, 0x204);
}

#[test]
fn skip_if_key_not_pressed() {
    let mut not_pressed = load(&[0x60, 0x05, 0xE0, 0xA1]);
    not_pressed.step().unwrap();
    not_pressed.step().unwrap();
    assert_eq!(not_pressed.pc, 0x206);

    let mut pressed = load(&[0x60, 0x05, 0xE0, 0xA1]);
    pressed.set_key(5, true);
    pressed.step().unwrap();
    pressed.step().unwrap();
    assert_eq!(pressed.pc, 0x204);
}

#[test]
fn released_key_is_not_pressed() {
    let mut machine = load(&[0x60, 0x05, 0xE0, 0x9E]);
    machine.set_key(5, true);
    machine.set_key(5, false);
    machine.step().unwrap();
    machine.step().unwrap();
    assert_eq!(machine.pc, 0x204);
}

#[test]
fn wait_for_key_blocks_until_a_key_is_pressed_and_released() {
    // F00A: wait for a key, store it in V0
    let mut machine = load(&[0xF0, 0x0A]);

    // no key: keeps re-running the same instruction
    for _ in 0..3 {
        machine.step().unwrap();
        assert_eq!(machine.pc, 0x200);
    }

    // Original COSMAC VIP quirk: the key counts on release, not on press.
    machine.set_key(7, true);
    machine.step().unwrap();
    assert_eq!(machine.pc, 0x200, "still waiting while the key is held");

    machine.set_key(7, false);
    machine.step().unwrap();
    assert_eq!(machine.pc, 0x202);
    assert_eq!(machine.v[0], 7);
}

#[test]
fn timers_keep_ticking_while_waiting_for_a_key() {
    // V0 = 3; delay = V0; F10A (wait); after 3 ticks the delay is 0
    let mut machine = load(&[0x60, 0x03, 0xF0, 0x15, 0xF1, 0x0A, 0xF2, 0x07]);
    machine.step().unwrap();
    machine.step().unwrap();
    for _ in 0..3 {
        machine.step().unwrap(); // waiting
        machine.tick_timers();
    }
    machine.set_key(1, true);
    machine.step().unwrap();
    machine.set_key(1, false);
    machine.step().unwrap(); // key released -> V1 = 1
    machine.step().unwrap(); // V2 = delay
    assert_eq!(machine.v[1], 1);
    assert_eq!(machine.v[2], 0);
}
