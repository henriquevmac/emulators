# Emulators

Learn how a CPU and its hardware work by writing emulators in Rust: CHIP-8 first (a weekend, to learn the fetch/decode/execute loop), then a Game Boy (DMG) emulator that boots real games.

- Linear: https://linear.app/henriquevmac-personal/project/emulators-550acc1b6492
- Notes: ~/Documents/notes/Side Projects/Emulators/ (_Overview.md, Dev Log.md, Resources.md)

## How we work (learning project)

- **The user writes all the code.** Do not edit source files unless explicitly asked. Explain, review, and give small snippets — not full solutions.
- **"check"** means: read the current sources, build, run the tests, and review against the current milestone. Report findings most-important-first with `file:line`, explain *why* each one is a problem, and say clearly what is already correct.
- **Verify behaviour, don't guess:** run the program / test ROMs / smoke tests in the session scratchpad directory, never in the project directory. Clean up any processes a test leaves behind.
- **Concept questions** get a direct explanation tied to this codebase.
- **Milestones:** when one is done, offer to mark its Linear issues done and to append a `## YYYY-MM-DD` entry to `Dev Log.md`.
- **Commits:** suggest good commit points; never commit yourself.
