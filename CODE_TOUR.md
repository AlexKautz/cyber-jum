# Cyber Jum — Code Tour

A guided map of the codebase. The game is written in Rust on top of the
[agb](https://agbrs.dev) crate (v0.24), which wraps the Game Boy Advance
hardware in a safe, game-engine-flavored API.

## Thirty seconds of GBA background

Knowing four hardware facts makes the whole codebase legible:

1. **The screen is 240x160** and redraws at ~60 fps. All game logic runs once
   per frame; `frame.commit()` waits for the vertical blank.
2. **Backgrounds are grids of 8x8 tiles**, up to four layers deep, each with a
   priority (P0 draws on top, P3 at the back). Big maps scroll by sliding a
   tile window around — agb's `InfiniteScrolledMap` handles that.
3. **Sprites ("objects") are small movable images** (our sprites are 16x16,
   the hearts 8x8, the flag 16x32). You re-declare what objects you want
   visible every frame.
4. **Everything is paletted**: 16 colors per palette, which is why the asset
   generator enforces ≤15 colors + transparency per image.

There is no operating system and no `std` — hence `#![no_std]` and the
`alloc` crate for `Vec`/`String` (agb provides the allocator).

## File-by-file

### [src/main.rs](src/main.rs) — entry point and scene flow

Owns the hardware handles (graphics, mixer, save, buttons) and runs the
top-level state machine: title screen → overworld → platformer challenges →
ending → back to title. Each scene is a function that loops internally and
returns an enum saying what happened (`WorldResult`, `LevelResult`, ...).
The flow diagram is in the module doc comment at the top of the file.

### [src/assets.rs](src/assets.rs) — every asset import

All `include_aseprite!` (sprites), `include_background_gfx!` (tile sheets),
`include_wav!` (audio) and `include_font!` calls live here, so the rest of
the code refers to assets by name (`assets::HERO_TOPDOWN`,
`assets::MUSIC_TITLE`, ...). agb converts the PNG/WAV files into GBA-native
data at compile time. Comments document what each frame index means.

### [src/world_map.rs](src/world_map.rs) — the overworld map data

The forest road as ASCII art: 15x64 cells of 16x16 pixels, one character per
cell (`T` tree, `g` grass, `.` road, ...). Below the map sit the functions
that interpret it: `walkable()` for collision and `tile_at()` which expands
each cell into a 2x2 quad of 8x8 tiles (with deterministic grass variation so
the ground doesn't look like wallpaper). Challenger positions and the hero
spawn are constants here too.

### [src/overworld.rs](src/overworld.rs) — top-down gameplay

A `Mode` enum is the heart of it: `Walk`, `Alert` (the "!" moment when a
challenger spots you), `Approach` (they walk over), `Talk`, `Paused`,
`Settings`, `Toast`. One mode runs per frame; rendering happens at the bottom
of the loop regardless of mode. Movement is pixel-based with per-axis
collision against the cell grid (`can_stand`), so you slide along walls.
Hold B to run. START opens the pause menu, where SAVE GAME writes to SRAM.

### [src/levels.rs](src/levels.rs) — the five platformer levels

Each level is 60x10 cells of ASCII art plus its challenger's intro/outro
dialogue. The legend and the level-design rules (max pit width, max climb
height — derived from the jump physics) are in the header comment. The maps
are validated by asserts at load time, so a broken edit fails loudly.

### [src/platformer.rs](src/platformer.rs) — side-scrolling gameplay

The Mario-style engine:

- **Physics** — fixed-point numbers (`Num<i32, 8>`, no FPU on the GBA!);
  constants at the top of the file are tuned together with the level rules.
- **Hero** (`Hero::update`) — run acceleration/friction, gravity, and the
  quality-of-life trio: coyote time, jump buffering, variable jump height.
- **Collision** (`move_and_collide`) — axis-by-axis against the cell grid;
  `=` platforms are one-way (land on top, jump through from below).
- **Bugs** (`update_bugs`) — patrol and turn at walls/ledges; stomping
  squashes them (hold A while stomping for a higher bounce), side contact
  kills.
- **Phases** — `Intro` banner → `Play` → `Dying`/`Won`, with three lives
  shown as heart objects.
- **Backgrounds** — a parallax synthwave backdrop (`make_backdrop`) behind an
  `InfiniteScrolledMap` of the terrain (`terrain_tile` picks each 8x8 tile,
  e.g. block tops vs fills, platform end caps).

### [src/text.rs](src/text.rs) — dialogue boxes, banners, menus

UI building blocks shared by every scene, drawn with agb's font renderer on
transparent background layers above a dark panel layer:

- `Dialogue` — bottom-screen box with a typewriter effect (A skips, A turns
  pages, blips as letters appear).
- `Banner` — static centred text ("GAME SAVED!", level intros, the ending).
- `Menu` — vertical list with a `>` cursor; arrow keys move, left/right
  adjust values, A selects, B backs out.
- `SettingsMenu` — the music/SFX volume editor used by both the title screen
  and the pause menu; changes apply (and are audible) immediately.

### [src/audio.rs](src/audio.rs) — mixer wrapper

`Audio` wraps agb's software mixer: looping high-priority music channel,
fire-and-forget SFX channels, and the 0–10 volume settings (applied live to
the playing music). `audio.frame()` must run every frame — each scene's loop
does so.

### [src/save.rs](src/save.rs) — SRAM saves

`SaveData` (hero position, beaten challengers, volumes) is serialized with
serde into cartridge SRAM via agb's `SaveSlotManager`, guarded by a magic
version string. Emulators persist SRAM as a `.sav` file next to the ROM.

## Asset pipeline

PNG art and WAV audio are generated by Python scripts in
[tools/asset_gen/](tools/asset_gen/) — sprites are ASCII pixel art, melodies
are note lists. See [assets/README.md](assets/README.md) for the inventory
and regeneration commands.

## Build configuration

- [Cargo.toml](Cargo.toml) — agb + serde; aggressive optimization even in dev
  (the GBA CPU is 16.78 MHz).
- [rust-toolchain.toml](rust-toolchain.toml) — pins nightly (the GBA target
  is tier 3 and needs `build-std`).
- [.cargo/config.toml](.cargo/config.toml) — target `thumbv4t-none-eabi`,
  agb's linker script, and `cargo run` wired to the mGBA emulator.

See [BUILDING.md](BUILDING.md) to compile and [TESTING.md](TESTING.md) to play.
