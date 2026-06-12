# Cyber Jum Game — CLAUDE.md

A demo Game Boy Advance game written in Rust using the [agb](https://agbrs.dev) crate
(v0.24). The game mixes a Pokémon-style top-down overworld with Mario-style 2D
platformer challenge levels ("Cyber Jump!"). Full design spec lives in
`instructions.txt`.

## Environment quirks (macOS, Homebrew)

- Rust is installed via the **Homebrew `rustup` formula**. The proxy binaries
  (`cargo`, `rustc`, `rustup`) live in `/opt/homebrew/opt/rustup/bin`, which is
  **not** on the default PATH for non-interactive shells. Prefix commands with:
  `export PATH="/opt/homebrew/opt/rustup/bin:$PATH"`
- The project pins **nightly** Rust via `rust-toolchain.toml` (needed for
  `build-std` on the tier-3 `thumbv4t-none-eabi` target). `rust-src` component
  must be installed on nightly.
- `agb-gbafix` (installed via `cargo install agb-gbafix`) converts the built ELF
  into a runnable `.gba` ROM with a corrected header.
- mGBA (`brew install mgba`) is the emulator for manual testing.
- Python work uses **uv** (`uv run ...`), never bare pip/python.

## Build commands

```sh
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo build --release                      # builds the ELF for thumbv4t-none-eabi
agb-gbafix target/thumbv4t-none-eabi/release/cyber_jum -o cyber_jum.gba
```

Target/linker config is in `.cargo/config.toml` (build-std core+alloc,
`-Tgba.ld`, `target-cpu=arm7tdmi`). Run in emulator: `mgba-qt cyber_jum.gba`
(or open in the mGBA app).

## Asset pipeline

All art (PNG) and audio (WAV) is **generated procedurally** by Python scripts in
`tools/asset_gen/` (a uv project) into `assets/`:

```sh
cd tools/asset_gen && uv run python generate_art.py && uv run python generate_audio.py
```

- Sprites: PNG horizontal strips imported with agb's `include_aseprite!`
  (which accepts `.png` since agb 0.24). Keep each image ≤16 colors —
  GBA sprites/backgrounds are 4bpp.
- Backgrounds: PNG tilesets imported with `include_background_gfx!`.
- Audio: mono WAV imported with `include_wav!`; music is looped WAV played
  through agb's software mixer (no tracker files).
- **Regenerate assets via the scripts; never hand-edit the PNG/WAV outputs.**

## Project layout

- `src/` — game code (see `CODE_TOUR.md` for a guided map)
- `assets/` — generated art + audio (review-friendly formats: PNG/WAV)
- `tools/asset_gen/` — uv Python project that generates `assets/`
- `BUILDING.md` — step-by-step compile instructions from a clean Mac
- `TESTING.md` — emulator testing instructions
- `AI_PROVENANCE.md` — which AI model generated this code

## Process rules from instructions.txt

- Assets must be reviewable by the user (PNG/WAV) **before** game code is written.
- Code must prioritize readability; document non-obvious GBA constraints inline.
- Always actually compile after changes (`cargo build --release`) — the
  maintainer cannot run the emulator loop for every change, so a clean build +
  ROM analysis is the verification bar.
