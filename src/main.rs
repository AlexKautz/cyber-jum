//! Cyber Jum — a Game Boy Advance demo game built with the agb crate.
//!
//! A top-down overworld (walk up the forest road) is gated by five
//! challengers, each guarding a side-on "Cyber Jump" platformer level.
//! Beat all five to win.  See `CODE_TOUR.md` for a guided map of the code.
//!
//! High-level flow, owned by `main`:
//!
//!   title ──new game / continue──▶ overworld ──challenged──▶ platformer
//!     ▲                                │  ▲                      │
//!     └────────── quit / ending ◀─────┘  └──── won / lost ◀─────┘

#![no_std]
#![no_main]

extern crate alloc;

mod assets;
mod audio;
mod levels;
mod overworld;
mod platformer;
mod save;
mod text;
mod title;
mod world_map;

use agb::display::Rgb15;
use agb::input::ButtonController;
use agb::sound::mixer::Frequency;

use audio::Audio;
use overworld::{EntryEvent, WorldResult, WorldState};
use platformer::LevelResult;
use save::SaveData;
use title::TitleResult;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut input = ButtonController::new();
    let mut save = save::SaveFile::new(&mut gba.save);

    // Background palettes for all tile art, plus white-on-shadow text
    // colours in the slot reserved for the font renderer.
    gfx.set_background_palettes(assets::PALETTES);
    gfx.set_background_palette_colour(assets::TEXT_PALETTE.into(), 1, Rgb15::WHITE);
    gfx.set_background_palette_colour(assets::TEXT_PALETTE.into(), 2, Rgb15::BLACK);

    // Volumes persist in the save file; fall back to comfortable defaults.
    let initial = save.load().unwrap_or_default();
    let mut audio = Audio::new(
        gba.mixer.mixer(Frequency::Hz18157),
        initial.music_volume,
        initial.sfx_volume,
    );

    loop {
        // ----- Title screen -------------------------------------------------
        let mut state = match title::run(&mut gfx, &mut audio, &mut input, &mut save) {
            TitleResult::NewGame => WorldState::from_save(&SaveData::default()),
            TitleResult::Continue(data) => {
                audio.set_music_volume(data.music_volume);
                audio.set_sfx_volume(data.sfx_volume);
                WorldState::from_save(&data)
            }
        };

        // ----- One play-through ---------------------------------------------
        let mut entry_event = None;
        loop {
            match overworld::run(
                &mut gfx,
                &mut audio,
                &mut input,
                &mut save,
                &mut state,
                entry_event.take(),
            ) {
                WorldResult::Challenge(level) => {
                    entry_event = Some(
                        match platformer::run(&mut gfx, &mut audio, &mut input, level) {
                            LevelResult::Won => {
                                state.beaten[level] = true;
                                EntryEvent::WonLevel(level)
                            }
                            LevelResult::Lost => EntryEvent::LostLevel,
                        },
                    );
                }
                WorldResult::Finished => {
                    title::show_ending(&mut gfx, &mut audio, &mut input);
                    break;
                }
                WorldResult::QuitToTitle => break,
            }
        }
    }
}
