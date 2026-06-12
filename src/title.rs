//! The title screen (start / continue / settings) and the ending screen.

use agb::display::Priority;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use alloc::string::String;
use alloc::vec::Vec;

use crate::assets;
use crate::audio::Audio;
use crate::save::{SaveData, SaveFile};
use crate::text::{Banner, Menu, MenuAction, SettingsMenu};

pub enum TitleResult {
    NewGame,
    Continue(SaveData),
}

enum Mode {
    Menu(Menu),
    Settings(SettingsMenu),
}

pub fn run(
    gfx: &mut agb::display::Graphics,
    audio: &mut Audio,
    input: &mut ButtonController,
    save: &mut SaveFile,
) -> TitleResult {
    let mut background = RegularBackground::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    background.fill_with(&assets::title);

    let saved_game = save.load();

    let mut items: Vec<String> = alloc::vec![String::from("NEW GAME")];
    if saved_game.is_some() {
        items.push(String::from("CONTINUE"));
    }
    items.push(String::from("SETTINGS"));

    let mut mode = Mode::Menu(Menu::new("", items.clone(), 14));

    audio.play_music(assets::MUSIC_TITLE);

    loop {
        input.update();

        match mode {
            Mode::Menu(ref mut menu) => match menu.update(input, audio) {
                MenuAction::Selected(0) => return TitleResult::NewGame,
                MenuAction::Selected(1) if saved_game.is_some() => {
                    return TitleResult::Continue(saved_game.unwrap());
                }
                MenuAction::Selected(_) => mode = Mode::Settings(SettingsMenu::new(audio)),
                _ => {}
            },
            Mode::Settings(ref mut menu) => {
                if menu.update(input, audio) {
                    mode = Mode::Menu(Menu::new("", items.clone(), 14));
                }
            }
        }

        let mut frame = gfx.frame();
        background.show(&mut frame);
        match &mode {
            Mode::Menu(menu) => menu.show(&mut frame),
            Mode::Settings(menu) => menu.show(&mut frame),
        }

        audio.frame();
        frame.commit();
    }
}

/// Shown once the hero has beaten all five challengers and walked off the
/// top of the map.  Waits for A, then returns to the title screen.
pub fn show_ending(
    gfx: &mut agb::display::Graphics,
    audio: &mut Audio,
    input: &mut ButtonController,
) {
    let mut background = RegularBackground::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    background.fill_with(&assets::title);
    // Park the title art's sky over the whole screen by scrolling past the logo.
    background.set_scroll_pos(vec2(0, 0));

    let banner = Banner::new(
        "CONGRATULATIONS!\n\nYou beat all five CYBER JUMP challengers\nand the road north is open.\n\nTHE END\n\n(press A)",
        4,
        15,
    );

    audio.play_music(assets::MUSIC_TITLE);
    audio.sfx(assets::SFX_WIN_LEVEL);

    loop {
        input.update();
        if input.is_just_pressed(Button::A) {
            return;
        }
        let mut frame = gfx.frame();
        background.show(&mut frame);
        banner.show(&mut frame);
        audio.frame();
        frame.commit();
    }
}
