//! Tests for milestone C3: the rest of the CHIP-8 instruction set.
//!
//! Behaviour follows the original CHIP-8 on the COSMAC VIP — the "CHIP-8"
//! choice in Timendus' `5-quirks.ch8`. Tests that depend on that choice are in
//! the "quirks" section at the bottom.
//!
//! Every test builds a tiny ROM, runs it with `step()`, and checks registers,
//! memory or `pc`. ROM bytes are written as instruction pairs, e.g. `0x60, 0x42`
//! is `6042` (V0 = 0x42).

use super::*;

/// Loads `rom` and executes `steps` instructions, failing on any `Err`.
fn run(rom: &[u8], steps: usize) -> Chip8 {
    let mut machine = Chip8::from_rom(rom).unwrap();
    for _ in 0..steps {
        machine.step().unwrap();
    }
    machine
}

// ---------------------------------------------------------------------------
// Subroutines: 2NNN, 00EE
// ---------------------------------------------------------------------------

#[test]
fn call_pushes_return_address_and_jumps() {
    let machine = run(&[0x23, 0x00], 1); // 2300: call 0x300
    assert_eq!(machine.pc, 0x300);
    assert_eq!(machine.stack, vec![0x202]); // the instruction after the call
}

#[test]
fn call_then_return() {
    let machine = run(
        &[
            0x22, 0x06, // 0x200: call 0x206
            0x61, 0x55, // 0x202: V1 = 0x55   (runs after the return)
            0x12, 0x04, // 0x204: jump to self
            0x60, 0x42, // 0x206: V0 = 0x42   (subroutine body)
            0x00, 0xEE, // 0x208: return
        ],
        4,
    );
    assert_eq!(machine.v[0], 0x42);
    assert_eq!(machine.v[1], 0x55);
    assert_eq!(machine.pc, 0x204);
    assert!(machine.stack.is_empty());
}

#[test]
fn nested_calls_return_in_order() {
    let machine = run(
        &[
            0x22, 0x04, // 0x200: call A (0x204)
            0x12, 0x02, // 0x202: jump to self
            0x22, 0x08, // 0x204: A: call B (0x208)
            0x00, 0xEE, // 0x206: A: return
            0x60, 0x07, // 0x208: B: V0 = 7
            0x00, 0xEE, // 0x20A: B: return
        ],
        5,
    );
    assert_eq!(machine.v[0], 7);
    assert_eq!(machine.pc, 0x202);
    assert!(machine.stack.is_empty());
}

#[test]
fn return_with_empty_stack_is_an_error() {
    let mut machine = Chip8::from_rom(&[0x00, 0xEE]).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn stack_holds_16_calls_and_the_17th_is_an_error() {
    // 2200 calls itself forever
    let mut machine = Chip8::from_rom(&[0x22, 0x00]).unwrap();
    for depth in 1..=16 {
        assert!(machine.step().is_ok(), "call {depth} should fit");
    }
    assert!(machine.step().is_err(), "call 17 should overflow");
}

// ---------------------------------------------------------------------------
// Skips: 3XNN, 4XNN, 5XY0, 9XY0 — "skip" means pc moves past one more instruction
// ---------------------------------------------------------------------------

#[test]
fn skip_if_vx_equals_nn() {
    let taken = run(&[0x60, 0x42, 0x30, 0x42], 2); // V0 = 42; skip if V0 == 42
    assert_eq!(taken.pc, 0x206);
    let not_taken = run(&[0x60, 0x41, 0x30, 0x42], 2);
    assert_eq!(not_taken.pc, 0x204);
}

#[test]
fn skip_if_vx_not_equal_nn() {
    let taken = run(&[0x60, 0x41, 0x40, 0x42], 2); // V0 = 41; skip if V0 != 42
    assert_eq!(taken.pc, 0x206);
    let not_taken = run(&[0x60, 0x42, 0x40, 0x42], 2);
    assert_eq!(not_taken.pc, 0x204);
}

#[test]
fn skip_if_vx_equals_vy() {
    let taken = run(&[0x60, 0x05, 0x61, 0x05, 0x50, 0x10], 3); // skip if V0 == V1
    assert_eq!(taken.pc, 0x208);
    let not_taken = run(&[0x60, 0x05, 0x61, 0x06, 0x50, 0x10], 3);
    assert_eq!(not_taken.pc, 0x206);
}

#[test]
fn skip_if_vx_not_equal_vy() {
    let taken = run(&[0x60, 0x05, 0x61, 0x06, 0x90, 0x10], 3); // skip if V0 != V1
    assert_eq!(taken.pc, 0x208);
    let not_taken = run(&[0x60, 0x05, 0x61, 0x05, 0x90, 0x10], 3);
    assert_eq!(not_taken.pc, 0x206);
}

// ---------------------------------------------------------------------------
// Register operations: 8XY0 - 8XY7 (8XY6 / 8XYE are under quirks)
// Setup is always V0 (X) and V1 (Y), then one 8-instruction.
// ---------------------------------------------------------------------------

/// Runs `V0 = vx; V1 = vy; <op>` and returns the machine.
fn alu(vx: u8, vy: u8, op: [u8; 2]) -> Chip8 {
    run(&[0x60, vx, 0x61, vy, op[0], op[1]], 3)
}

#[test]
fn set_vx_to_vy() {
    let machine = alu(0x00, 0x33, [0x80, 0x10]);
    assert_eq!(machine.v[0], 0x33);
    assert_eq!(machine.v[1], 0x33); // VY unchanged
}

#[test]
fn or() {
    assert_eq!(alu(0xF0, 0x0F, [0x80, 0x11]).v[0], 0xFF);
}

#[test]
fn and() {
    assert_eq!(alu(0xF0, 0x3C, [0x80, 0x12]).v[0], 0x30);
}

#[test]
fn xor() {
    assert_eq!(alu(0xF0, 0x3C, [0x80, 0x13]).v[0], 0xCC);
}

#[test]
fn add_without_carry() {
    let machine = alu(0x10, 0x20, [0x80, 0x14]);
    assert_eq!(machine.v[0], 0x30);
    assert_eq!(machine.v[0xF], 0);
}

#[test]
fn add_with_carry_wraps_and_sets_vf() {
    let machine = alu(0xFF, 0x02, [0x80, 0x14]);
    assert_eq!(machine.v[0], 0x01);
    assert_eq!(machine.v[0xF], 1);
}

#[test]
fn sub_vx_minus_vy_without_borrow() {
    let machine = alu(0x30, 0x10, [0x80, 0x15]);
    assert_eq!(machine.v[0], 0x20);
    assert_eq!(machine.v[0xF], 1); // VF = 1 means "no borrow"
}

#[test]
fn sub_vx_minus_vy_with_borrow() {
    let machine = alu(0x10, 0x30, [0x80, 0x15]);
    assert_eq!(machine.v[0], 0xE0);
    assert_eq!(machine.v[0xF], 0);
}

#[test]
fn sub_equal_values_is_no_borrow() {
    let machine = alu(0x42, 0x42, [0x80, 0x15]);
    assert_eq!(machine.v[0], 0x00);
    assert_eq!(machine.v[0xF], 1);
}

#[test]
fn sub_vy_minus_vx_without_borrow() {
    let machine = alu(0x10, 0x30, [0x80, 0x17]); // V0 = V1 - V0
    assert_eq!(machine.v[0], 0x20);
    assert_eq!(machine.v[0xF], 1);
}

#[test]
fn sub_vy_minus_vx_with_borrow() {
    let machine = alu(0x30, 0x10, [0x80, 0x17]);
    assert_eq!(machine.v[0], 0xE0);
    assert_eq!(machine.v[0xF], 0);
}

/// When VF itself is the target (X = F), the flag must win: it is written
/// after the result. Tested here with 8XY4 and 8XY5.
#[test]
fn flag_is_written_after_the_result() {
    // VF = 0xFF, V1 = 0x03; VF += V1 -> result 0x02, carry 1
    let add = run(&[0x6F, 0xFF, 0x61, 0x03, 0x8F, 0x14], 3);
    assert_eq!(add.v[0xF], 1);
    // VF = 0x10, V1 = 0x01; VF -= V1 -> result 0x0F, no borrow 1
    let sub = run(&[0x6F, 0x10, 0x61, 0x01, 0x8F, 0x15], 3);
    assert_eq!(sub.v[0xF], 1);
}

// ---------------------------------------------------------------------------
// BNNN, CXNN
// ---------------------------------------------------------------------------

#[test]
fn random_is_masked_by_nn() {
    // V0 = 0xFF, then C000: random & 0x00 must be 0
    assert_eq!(run(&[0x60, 0xFF, 0xC0, 0x00], 2).v[0], 0);
    // C00F: result must fit in the low nibble
    for _ in 0..100 {
        assert!(run(&[0xC0, 0x0F], 1).v[0] <= 0x0F);
    }
}

#[test]
fn random_is_not_constant() {
    let mut seen = std::collections::HashSet::new();
    for _ in 0..200 {
        seen.insert(run(&[0xC0, 0xFF], 1).v[0]);
    }
    assert!(seen.len() > 1, "CXFF returned the same value 200 times");
}

// ---------------------------------------------------------------------------
// FX instructions (timers' FX07/FX15 here; ticking and sound are in c3_io_tests)
// ---------------------------------------------------------------------------

#[test]
fn delay_timer_set_and_read_back() {
    // V0 = 0x42; delay = V0; V1 = delay  (no tick happens between steps)
    let machine = run(&[0x60, 0x42, 0xF0, 0x15, 0xF1, 0x07], 3);
    assert_eq!(machine.v[1], 0x42);
}

#[test]
fn add_vx_to_i() {
    let machine = run(&[0xA1, 0x00, 0x60, 0x10, 0xF0, 0x1E], 3); // I = 0x100; I += V0
    assert_eq!(machine.i, 0x110);
}

#[test]
fn font_character_address() {
    // V0 = 0xA; I = address of font "A" = 0x50 + 0xA * 5
    let machine = run(&[0x60, 0x0A, 0xF0, 0x29], 2);
    assert_eq!(machine.i, 0x50 + 0xA * 5);
}

#[test]
fn font_character_uses_low_nibble_only() {
    let machine = run(&[0x60, 0x1A, 0xF0, 0x29], 2); // 0x1A -> character A
    assert_eq!(machine.i, 0x50 + 0xA * 5);
}

#[test]
fn bcd_of_vx() {
    // V0 = 156; I = 0x300; store BCD at I, I+1, I+2
    let machine = run(&[0x60, 156, 0xA3, 0x00, 0xF0, 0x33], 3);
    assert_eq!(&machine.memory[0x300..0x303], &[1, 5, 6]);
    assert_eq!(machine.i, 0x300); // FX33 leaves I alone

    let small = run(&[0x60, 7, 0xA3, 0x00, 0xF0, 0x33], 3);
    assert_eq!(&small.memory[0x300..0x303], &[0, 0, 7]);
}

#[test]
fn store_registers_v0_to_vx() {
    // V0..V2 = 1, 2, 3; V3 = 9 (must NOT be stored); I = 0x300; F255
    let machine = run(
        &[0x60, 1, 0x61, 2, 0x62, 3, 0x63, 9, 0xA3, 0x00, 0xF2, 0x55],
        6,
    );
    assert_eq!(&machine.memory[0x300..0x304], &[1, 2, 3, 0]);
}

#[test]
fn load_registers_v0_to_vx() {
    let machine = run(
        &[
            0xA2, 0x06, // 0x200: I = 0x206 (the data below)
            0xF2, 0x65, // 0x202: load V0..V2
            0x12, 0x04, // 0x204: jump to self
            0x11, 0x22, 0x33, 0x44, // 0x206: data
        ],
        2,
    );
    assert_eq!(&machine.v[0..4], &[0x11, 0x22, 0x33, 0x00]); // V3 untouched
}

// ---------------------------------------------------------------------------
// Quirks: original CHIP-8 (COSMAC VIP) behaviour.
// Later interpreters (CHIP-48 / SUPER-CHIP) differ here — see Langhoff's guide.
// ---------------------------------------------------------------------------

/// 8XY1 / 8XY2 / 8XY3 reset VF to 0 on the original.
#[test]
fn quirk_logic_ops_reset_vf() {
    for op in [0x11, 0x12, 0x13] {
        // VF = 5; V0 = 1; V1 = 2; 80 1op
        let machine = run(&[0x6F, 0x05, 0x60, 0x01, 0x61, 0x02, 0x80, op], 4);
        assert_eq!(machine.v[0xF], 0, "8XY{:X} should reset VF", op & 0xF);
    }
}

/// 8XY6: VX = VY >> 1 (VY is the source on the original), VF = bit shifted out.
#[test]
fn quirk_shift_right_uses_vy() {
    let machine = alu(0xFF, 0x05, [0x80, 0x16]); // 0b0101 >> 1
    assert_eq!(machine.v[0], 0x02);
    assert_eq!(machine.v[0xF], 1);

    let machine = alu(0xFF, 0x04, [0x80, 0x16]); // 0b0100 >> 1
    assert_eq!(machine.v[0], 0x02);
    assert_eq!(machine.v[0xF], 0);
}

/// 8XYE: VX = VY << 1, VF = bit shifted out.
#[test]
fn quirk_shift_left_uses_vy() {
    let machine = alu(0x00, 0x81, [0x80, 0x1E]); // 0b1000_0001 << 1
    assert_eq!(machine.v[0], 0x02);
    assert_eq!(machine.v[0xF], 1);

    let machine = alu(0x00, 0x40, [0x80, 0x1E]);
    assert_eq!(machine.v[0], 0x80);
    assert_eq!(machine.v[0xF], 0);
}

/// BNNN jumps to NNN + V0 (SUPER-CHIP's BXNN would use VX instead).
#[test]
fn quirk_jump_with_offset_uses_v0() {
    // V0 = 0x10; V3 = 0x50; B320 -> 0x320 + V0 = 0x330 (not 0x370)
    let machine = run(&[0x60, 0x10, 0x63, 0x50, 0xB3, 0x20], 3);
    assert_eq!(machine.pc, 0x330);
}

/// FX55 / FX65 leave I pointing past the last register (I += X + 1).
#[test]
fn quirk_store_and_load_increment_i() {
    let store = run(&[0xA3, 0x00, 0xF2, 0x55], 2);
    assert_eq!(store.i, 0x303);
    let load = run(&[0xA3, 0x00, 0xF2, 0x65], 2);
    assert_eq!(load.i, 0x303);
}
