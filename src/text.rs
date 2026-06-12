//! Text UI building blocks: dialogue boxes, banners and cursor menus.
//!
//! All text is drawn with agb's font renderer onto its own transparent
//! background layer, floating above a "panel" layer built from the dark
//! `ui_panel` tiles.  Components are immediate-mode-ish: call `update` once
//! per frame, then `show` while building the frame.

use agb::display::font::{AlignmentKind, Layout, LayoutSettings, RegularBackgroundTextRenderer};
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, Priority};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use alloc::string::String;
use alloc::vec::Vec;

use crate::assets;
use crate::audio::Audio;

/// Screen width in 8x8 tiles.
const SCREEN_TILES_X: i32 = 30;

/// Fill tile rows `top..=bottom` of a background with the panel box art.
fn make_panel(top: i32, bottom: i32) -> RegularBackground {
    let mut bg = RegularBackground::new(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    for y in top..=bottom {
        let tile = if y == top {
            1 // top border
        } else if y == bottom {
            2 // bottom border
        } else {
            0 // fill
        };
        for x in 0..SCREEN_TILES_X {
            bg.set_tile(
                vec2(x, y),
                &assets::panel.tiles,
                assets::panel.tile_settings[tile],
            );
        }
    }
    bg
}

fn empty_text_layer() -> RegularBackground {
    RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    )
}

// ---------------------------------------------------------------------------
// Dialogue: a bottom-of-screen box that types text out page by page.
// ---------------------------------------------------------------------------

/// Tile rows used by the dialogue box (the bottom 6 of the screen's 20).
const DIALOGUE_TOP_ROW: i32 = 14;
const DIALOGUE_BOTTOM_ROW: i32 = 19;

pub struct Dialogue {
    panel: RegularBackground,
    text: RegularBackground,
    renderer: RegularBackgroundTextRenderer,
    layout: Layout,
    pages: &'static [&'static str],
    next_page: usize,
    page_done: bool,
    tick: u32,
}

impl Dialogue {
    pub fn new(pages: &'static [&'static str]) -> Self {
        let mut dialogue = Self {
            panel: make_panel(DIALOGUE_TOP_ROW, DIALOGUE_BOTTOM_ROW),
            text: empty_text_layer(),
            renderer: RegularBackgroundTextRenderer::new((0, 0), assets::TEXT_PALETTE),
            layout: Self::layout_for(""),
            pages,
            next_page: 0,
            page_done: false,
            tick: 0,
        };
        dialogue.start_next_page();
        dialogue
    }

    fn layout_for(page: &'static str) -> Layout {
        Layout::new(
            page,
            &assets::FONT,
            &LayoutSettings::new().with_max_line_length(216),
        )
    }

    fn start_next_page(&mut self) {
        self.text = empty_text_layer();
        self.renderer = RegularBackgroundTextRenderer::new(
            (12, DIALOGUE_TOP_ROW * 8 + 7),
            assets::TEXT_PALETTE,
        );
        self.layout = Self::layout_for(self.pages[self.next_page]);
        self.next_page += 1;
        self.page_done = false;
    }

    /// Advance the typewriter; returns `true` once the player has read
    /// (and confirmed with A) every page.
    pub fn update(&mut self, input: &ButtonController, audio: &mut Audio) -> bool {
        self.tick += 1;

        if !self.page_done {
            if input.is_just_pressed(Button::A) {
                // Impatient player: reveal the rest of the page at once.
                for group in self.layout.by_ref() {
                    self.renderer.show(&mut self.text, &group);
                }
                self.page_done = true;
            } else if self.tick.is_multiple_of(2) {
                match self.layout.next() {
                    Some(group) => {
                        self.renderer.show(&mut self.text, &group);
                        if self.tick.is_multiple_of(8) {
                            audio.sfx(assets::SFX_TALK_BLIP);
                        }
                    }
                    None => self.page_done = true,
                }
            }
        } else if input.is_just_pressed(Button::A) {
            if self.next_page < self.pages.len() {
                audio.sfx(assets::SFX_MENU_MOVE);
                self.start_next_page();
            } else {
                return true;
            }
        }
        false
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.panel.show(frame);
        self.text.show(frame);
    }
}

// ---------------------------------------------------------------------------
// Banner: a panel with fully rendered (non-animated) centred text, used for
// level intros, "GAME SAVED!" toasts and the ending screen.
// ---------------------------------------------------------------------------

pub struct Banner {
    panel: RegularBackground,
    text: RegularBackground,
}

impl Banner {
    pub fn new(message: &str, top_row: i32, bottom_row: i32) -> Self {
        let mut text = empty_text_layer();
        let mut renderer =
            RegularBackgroundTextRenderer::new((0, top_row * 8 + 7), assets::TEXT_PALETTE);
        let layout = Layout::new(
            message,
            &assets::FONT,
            &LayoutSettings::new()
                .with_max_line_length(240)
                .with_alignment(AlignmentKind::Centre),
        );
        for group in layout {
            renderer.show(&mut text, &group);
        }
        Self {
            panel: make_panel(top_row, bottom_row),
            text,
        }
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.panel.show(frame);
        self.text.show(frame);
    }
}

// ---------------------------------------------------------------------------
// Menu: a vertical list with a ">" cursor.
// ---------------------------------------------------------------------------

pub enum MenuAction {
    None,
    /// A pressed on item `i`.
    Selected(usize),
    /// Left/right pressed on item `i` (for value sliders): -1 or +1.
    Adjusted(usize, i32),
    /// B pressed.
    Back,
}

pub struct Menu {
    title: &'static str,
    items: Vec<String>,
    pub cursor: usize,
    top_row: i32,
    panel: RegularBackground,
    text: RegularBackground,
}

impl Menu {
    pub fn new(title: &'static str, items: Vec<String>, top_row: i32) -> Self {
        // One text line per item (plus the title when present), roughly one
        // tile row each, plus a border row on each side.
        let lines = items.len() as i32 + if title.is_empty() { 0 } else { 1 };
        let bottom_row = top_row + lines + 2;
        let mut menu = Self {
            title,
            items,
            cursor: 0,
            top_row,
            panel: make_panel(top_row, bottom_row),
            text: empty_text_layer(),
        };
        menu.render_text();
        menu
    }

    /// Replace the menu's labels (used when a value like volume changes).
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.render_text();
    }

    fn render_text(&mut self) {
        self.text = empty_text_layer();
        let mut content = String::from(self.title);
        for (i, item) in self.items.iter().enumerate() {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(if i == self.cursor { "> " } else { "  " });
            content.push_str(item);
        }
        let mut renderer =
            RegularBackgroundTextRenderer::new((84, self.top_row * 8 + 7), assets::TEXT_PALETTE);
        let layout = Layout::new(
            &content,
            &assets::FONT,
            &LayoutSettings::new().with_max_line_length(156),
        );
        for group in layout {
            renderer.show(&mut self.text, &group);
        }
    }

    pub fn update(&mut self, input: &ButtonController, audio: &mut Audio) -> MenuAction {
        if input.is_just_pressed(Button::Up) && self.cursor > 0 {
            self.cursor -= 1;
            audio.sfx(assets::SFX_MENU_MOVE);
            self.render_text();
        }
        if input.is_just_pressed(Button::Down) && self.cursor + 1 < self.items.len() {
            self.cursor += 1;
            audio.sfx(assets::SFX_MENU_MOVE);
            self.render_text();
        }
        if input.is_just_pressed(Button::Left) {
            return MenuAction::Adjusted(self.cursor, -1);
        }
        if input.is_just_pressed(Button::Right) {
            return MenuAction::Adjusted(self.cursor, 1);
        }
        if input.is_just_pressed(Button::A) {
            audio.sfx(assets::SFX_MENU_SELECT);
            return MenuAction::Selected(self.cursor);
        }
        if input.is_just_pressed(Button::B) {
            return MenuAction::Back;
        }
        MenuAction::None
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.panel.show(frame);
        self.text.show(frame);
    }
}

// ---------------------------------------------------------------------------
// Settings: the music/sfx volume editor, reachable from both the title
// screen and the in-game pause menu.
// ---------------------------------------------------------------------------

pub struct SettingsMenu {
    menu: Menu,
}

fn volume_label(name: &str, volume: u8) -> String {
    let mut label = String::from(name);
    label.push(' ');
    for i in 0..crate::audio::MAX_VOLUME {
        label.push(if i < volume { '|' } else { '-' });
    }
    label
}

impl SettingsMenu {
    pub fn new(audio: &Audio) -> Self {
        Self {
            menu: Menu::new("SETTINGS", Self::labels(audio), 6),
        }
    }

    fn labels(audio: &Audio) -> Vec<String> {
        alloc::vec![
            volume_label("MUSIC", audio.music_volume()),
            volume_label("SOUND", audio.sfx_volume()),
            String::from("BACK"),
        ]
    }

    /// Returns `true` when the player exits the settings screen.
    pub fn update(&mut self, input: &ButtonController, audio: &mut Audio) -> bool {
        match self.menu.update(input, audio) {
            MenuAction::Adjusted(0, delta) => {
                let volume = (audio.music_volume() as i32 + delta).clamp(0, 10) as u8;
                audio.set_music_volume(volume);
                self.menu.set_items(Self::labels(audio));
            }
            MenuAction::Adjusted(1, delta) => {
                let volume = (audio.sfx_volume() as i32 + delta).clamp(0, 10) as u8;
                audio.set_sfx_volume(volume);
                self.menu.set_items(Self::labels(audio));
                // Play a blip so the new effect volume can be judged instantly.
                audio.sfx(assets::SFX_TALK_BLIP);
            }
            MenuAction::Selected(2) | MenuAction::Back => return true,
            _ => {}
        }
        false
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.menu.show(frame);
    }
}
