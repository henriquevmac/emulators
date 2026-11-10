# Emulators

Learning how a CPU and its hardware work by writing emulators in Rust: first a
CHIP-8 interpreter, then a Game Boy (DMG) emulator.

| Emulator | Status |
|---|---|
| **CHIP-8** | Complete instruction set, 60 Hz timers, keypad. Passes the Timendus test suite (see [below](#test-results)). |
| **Game Boy** | In progress: CPU (SM83). |

## Layout

A Cargo workspace with one crate per emulator:

```
chip8/      CHIP-8 interpreter
  src/lib.rs        the machine (CPU, memory, framebuffer, timers, keypad) — no window code
  src/main.rs       minifb window, keyboard mapping, frame loop
  src/c3_*tests.rs  instruction / timer / keypad tests
gameboy/    Game Boy emulator (work in progress)
roms/       test ROMs and games — not committed, see "ROMs"
```

Each emulator core is independent of the window library: `main.rs` reads the
framebuffer and feeds in key presses, so the cores can be unit-tested and run
headless.

## Requirements

- Rust **1.85** or newer (edition 2024)
- For the window: [`minifb`](https://crates.io/crates/minifb), pulled in by Cargo

## CHIP-8

### Run

```sh
cargo run --release -p chip8 -- roms/2-ibm-logo.ch8
```

The window is the 64×32 display scaled 16× (1024×512). The emulator runs 10
instructions per frame at 60 frames per second; the delay and sound timers tick
once per frame (60 Hz).

### Controls

The CHIP-8 hex keypad is mapped onto the left side of the keyboard:

```
CHIP-8 keypad      Keyboard
1 2 3 C            1 2 3 4
4 5 6 D            Q W E R
7 8 9 E            A S D F
A 0 B F            Z X C V
```

<kbd>Esc</kbd> or closing the window quits.

### Behaviour

Where CHIP-8 interpreters disagree, this one follows the **original CHIP-8 on
the COSMAC VIP**:

- `8XY1` / `8XY2` / `8XY3` reset VF to 0
- `8XY6` / `8XYE` shift **VY** into VX; VF gets the bit shifted out
- `BNNN` jumps to `NNN + V0`
- `FX55` / `FX65` leave I at `I + X + 1`
- `FX1E` does not touch VF
- `FX0A` waits for a key to be pressed **and released**
- Sprites are clipped at the screen edges; only the start position wraps

Not implemented yet: the *display wait* quirk (one draw per frame) and sound
output (the sound timer runs, but there is no beep).

### Test results

With the [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite):

| ROM | Result |
|---|---|
| `1-chip8-logo`, `2-ibm-logo` | ✓ |
| `3-corax+` | all ✓ |
| `4-flags` | all ✓ |
| `5-quirks` (CHIP-8) | 5 / 6 — display wait ✗ |
| `6-keypad` | ✓ |

## Game Boy

Work in progress. The plan:

1. **CPU** — all SM83 instructions, checked with Blargg's `cpu_instrs` (results
   over the serial port) and [Gameboy Doctor](https://github.com/robert/gameboy-doctor)
2. **Memory map, timers, interrupts**
3. **PPU (graphics)** — checked with `dmg-acid2`
4. **Joypad and cartridges** (MBC1 / MBC3 / MBC5, save RAM)

## ROMs

`roms/` is ignored by git; put ROMs there yourself. Only use test ROMs,
public-domain / homebrew ROMs, or dumps of cartridges you own.

- CHIP-8 test suite: the `.ch8` files in
  [Timendus/chip8-test-suite `bin/`](https://github.com/Timendus/chip8-test-suite/tree/main/bin)
- Game Boy CPU tests: the individual ROMs in Blargg's
  [`cpu_instrs/individual/`](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs/individual)
- Game Boy graphics test: [dmg-acid2](https://github.com/mattcurrie/dmg-acid2)

## Tests

```sh
cargo test --workspace     # all crates
cargo test -p chip8        # just CHIP-8
```

## References

- Tobias V. Langhoff — [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
  (some entries describe SUPER-CHIP, not the original)
- [Pan Docs](https://gbdev.io/pandocs/) — Game Boy hardware reference
- [gbdev.io opcode table](https://gbdev.io/gb-opcodes/optables/)
- [Decoding Gameboy Z80 opcodes](https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20opcodes.html)
