//! The top-down overworld: walk up the forest road, get challenged by the
//! five gatekeepers, pause to save or change settings.

use agb::display::object::Object;
use agb::display::tiled::{
    InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, TileFormat,
};
use agb::display::{GraphicsFrame, HEIGHT, Priority};
use agb::fixnum::{Vector2D, vec2};
use agb::input::{Button, ButtonController};
use alloc::string::String;

use crate::audio::Audio;
use crate::levels::LEVELS;
use crate::save::{SaveData, SaveFile};
use crate::text::{Banner, Dialogue, Menu, MenuAction, SettingsMenu};
use crate::{assets, world_map};

pub enum WorldResult {
    /// The hero accepted challenger `n`'s cyber jump.
    Challenge(usize),
    /// All five challengers beaten and the hero walked off the top.
    Finished,
    QuitToTitle,
}

/// What just happened before re-entering the overworld.
pub enum EntryEvent {
    WonLevel(usize),
    LostLevel,
}

/// The persistent bits of the overworld, shared with the save system.
pub struct WorldState {
    pub hero_pos: Vector2D<i32>,
    pub beaten: [bool; 5],
}

impl WorldState {
    pub fn from_save(data: &SaveData) -> Self {
        Self {
            hero_pos: vec2(data.hero_x, data.hero_y),
            beaten: data.beaten,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Facing {
    Down,
    Up,
    Side { left: bool },
}

/// What the overworld is currently doing.  Exactly one of these runs per
/// frame; the world only animates/accepts movement in `Walk`.
enum Mode {
    Walk,
    /// A challenger spotted the hero: "!" over their head.
    Alert {
        npc: usize,
        timer: u32,
    },
    /// The challenger walks over to the hero.
    Approach {
        npc: usize,
    },
    Talk {
        dialogue: Dialogue,
        then: AfterTalk,
    },
    Toast {
        banner: Banner,
        timer: u32,
    },
    Paused {
        menu: Menu,
    },
    Settings {
        menu: SettingsMenu,
    },
}

enum AfterTalk {
    StartChallenge(usize),
    Nothing,
}

const PAUSE_ITEMS: [&str; 4] = ["RESUME", "SAVE GAME", "SETTINGS", "QUIT TO TITLE"];

static LOST_PAGES: &[&str] = &[
    "You're out of lives! Everything goes dark...",
    "...and you wake up back at the start of the road. Try again!",
];

pub fn run(
    gfx: &mut agb::display::Graphics,
    audio: &mut Audio,
    input: &mut ButtonController,
    save: &mut SaveFile,
    state: &mut WorldState,
    entry: Option<EntryEvent>,
) -> WorldResult {
    let mut map = InfiniteScrolledMap::new(RegularBackground::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    ));

    // Challenger positions (sprite centres).  They normally stand at their
    // map post; during an encounter (or a post-win chat) they step out.
    let mut npc_pos: [Vector2D<i32>; 5] =
        world_map::NPC_CELLS.map(|(cx, cy)| vec2(cx * 16 + 8, cy * 16 + 8));

    let mut facing = Facing::Up;
    let mut walking;
    let mut tick = 0u32;

    let mut mode = match entry {
        Some(EntryEvent::WonLevel(npc)) => {
            // The beaten challenger comes over to congratulate the hero.
            npc_pos[npc] = state.hero_pos + vec2(18, 0);
            Mode::Talk {
                dialogue: Dialogue::new(LEVELS[npc].outro),
                then: AfterTalk::Nothing,
            }
        }
        Some(EntryEvent::LostLevel) => {
            state.hero_pos = world_map::HERO_START;
            Mode::Talk {
                dialogue: Dialogue::new(LOST_PAGES),
                then: AfterTalk::Nothing,
            }
        }
        None => Mode::Walk,
    };

    audio.play_music(assets::MUSIC_OVERWORLD);

    loop {
        input.update();
        tick = tick.wrapping_add(1);
        walking = false;

        match mode {
            Mode::Walk => {
                walking = move_hero(state, &npc_pos, input, &mut facing);

                // Beaten challengers wander back to their roadside posts.
                for (npc, pos) in npc_pos.iter_mut().enumerate() {
                    if state.beaten[npc] {
                        let (cx, cy) = world_map::NPC_CELLS[npc];
                        let delta = vec2(cx * 16 + 8, cy * 16 + 8) - *pos;
                        pos.x += delta.x.signum();
                        if delta.x == 0 {
                            pos.y += delta.y.signum();
                        }
                    }
                }

                if input.is_just_pressed(Button::Start) {
                    audio.sfx(assets::SFX_MENU_SELECT);
                    mode = Mode::Paused {
                        menu: Menu::new("PAUSED", PAUSE_ITEMS.map(String::from).into(), 5),
                    };
                } else if let Some(npc) = spotted_by(state) {
                    audio.sfx(assets::SFX_TALK_BLIP);
                    mode = Mode::Alert { npc, timer: 45 };
                } else if state.beaten.iter().all(|&b| b)
                    && state.hero_pos.y / 16 <= world_map::GOAL_ROW
                {
                    return WorldResult::Finished;
                }
            }
            Mode::Alert { npc, ref mut timer } => {
                *timer -= 1;
                if *timer == 0 {
                    mode = Mode::Approach { npc };
                }
            }
            Mode::Approach { npc } => {
                // Walk to just beside the hero, x first, then y.
                let side = if npc_pos[npc].x >= state.hero_pos.x {
                    1
                } else {
                    -1
                };
                let target = state.hero_pos + vec2(side * 18, 0);
                let delta = target - npc_pos[npc];
                npc_pos[npc].x += delta.x.signum();
                if delta.x == 0 {
                    npc_pos[npc].y += delta.y.signum();
                }
                if delta == vec2(0, 0) {
                    mode = Mode::Talk {
                        dialogue: Dialogue::new(LEVELS[npc].intro),
                        then: AfterTalk::StartChallenge(npc),
                    };
                }
            }
            Mode::Talk {
                ref mut dialogue,
                ref then,
            } => {
                if dialogue.update(input, audio) {
                    match then {
                        AfterTalk::StartChallenge(npc) => return WorldResult::Challenge(*npc),
                        AfterTalk::Nothing => mode = Mode::Walk,
                    }
                }
            }
            Mode::Toast {
                banner: _,
                ref mut timer,
            } => {
                *timer -= 1;
                if *timer == 0 {
                    mode = Mode::Walk;
                }
            }
            Mode::Paused { ref mut menu } => match menu.update(input, audio) {
                MenuAction::Selected(0) | MenuAction::Back => mode = Mode::Walk,
                MenuAction::Selected(1) => {
                    save.store(&SaveData {
                        hero_x: state.hero_pos.x,
                        hero_y: state.hero_pos.y,
                        beaten: state.beaten,
                        music_volume: audio.music_volume(),
                        sfx_volume: audio.sfx_volume(),
                    });
                    audio.sfx(assets::SFX_SAVE_CHIME);
                    mode = Mode::Toast {
                        banner: Banner::new("GAME SAVED!", 8, 11),
                        timer: 80,
                    };
                }
                MenuAction::Selected(2) => {
                    mode = Mode::Settings {
                        menu: SettingsMenu::new(audio),
                    }
                }
                MenuAction::Selected(3) => return WorldResult::QuitToTitle,
                _ => {}
            },
            Mode::Settings { ref mut menu } => {
                if menu.update(input, audio) {
                    mode = Mode::Walk;
                }
            }
        }

        // Camera: vertical follow, clamped to the map.
        let camera = vec2(
            0,
            (state.hero_pos.y - HEIGHT / 2).clamp(0, world_map::HEIGHT_PX - HEIGHT),
        );
        map.set_scroll_pos(camera, |pos| {
            (
                &assets::topdown.tiles,
                assets::topdown.tile_settings[world_map::tile_at(pos.x, pos.y)],
            )
        });

        let mut frame = gfx.frame();
        map.show(&mut frame);

        show_hero(state.hero_pos - camera, facing, walking, tick, &mut frame);
        for (npc, pos) in npc_pos.iter().enumerate() {
            let screen = *pos - camera;
            if (-16..HEIGHT + 16).contains(&screen.y) {
                Object::new(assets::NPCS.sprite(npc))
                    .set_priority(Priority::P2)
                    .set_pos(screen - vec2(8, 8))
                    .show(&mut frame);
                if matches!(mode, Mode::Alert { npc: n, .. } if n == npc) {
                    Object::new(assets::UI.sprite(2))
                        .set_priority(Priority::P2)
                        .set_pos(screen - vec2(4, 22))
                        .show(&mut frame);
                }
            }
        }

        match &mode {
            Mode::Talk { dialogue, .. } => dialogue.show(&mut frame),
            Mode::Toast { banner, .. } => banner.show(&mut frame),
            Mode::Paused { menu } => menu.show(&mut frame),
            Mode::Settings { menu } => menu.show(&mut frame),
            _ => {}
        }

        audio.frame();
        frame.commit();
    }
}

/// Move the hero from d-pad input (B runs); returns whether they moved.
fn move_hero(
    state: &mut WorldState,
    npc_pos: &[Vector2D<i32>; 5],
    input: &ButtonController,
    facing: &mut Facing,
) -> bool {
    let direction = input.vector::<i32>();
    if direction == vec2(0, 0) {
        return false;
    }

    *facing = match (direction.x, direction.y) {
        (_, 1) => Facing::Down,
        (_, -1) => Facing::Up,
        (x, _) => Facing::Side { left: x < 0 },
    };

    let speed = if input.is_pressed(Button::B) { 2 } else { 1 };
    for _ in 0..speed {
        // Per-axis movement so walls slide instead of stick.
        let step_x = vec2(direction.x, 0);
        if can_stand(state.hero_pos + step_x, npc_pos) {
            state.hero_pos += step_x;
        }
        let step_y = vec2(0, direction.y);
        if can_stand(state.hero_pos + step_y, npc_pos) {
            state.hero_pos += step_y;
        }
    }
    true
}

/// Collision for the hero's feet (the lower half of the sprite), against
/// both the map and the challengers standing around.
fn can_stand(centre: Vector2D<i32>, npc_pos: &[Vector2D<i32>; 5]) -> bool {
    for (dx, dy) in [(-5, 0), (5, 0), (-5, 7), (5, 7)] {
        let cell = vec2(
            (centre.x + dx).div_euclid(16),
            (centre.y + dy).div_euclid(16),
        );
        if !world_map::walkable(cell.x, cell.y) {
            return false;
        }
        if npc_pos
            .iter()
            .any(|npc| vec2(npc.x.div_euclid(16), npc.y.div_euclid(16)) == cell)
        {
            return false;
        }
    }
    true
}

/// An unbeaten challenger notices the hero when they walk near their row.
fn spotted_by(state: &WorldState) -> Option<usize> {
    (0..5).find(|&npc| {
        !state.beaten[npc]
            && (state.hero_pos.y - (world_map::NPC_CELLS[npc].1 * 16 + 8)).abs() <= 24
    })
}

fn show_hero(
    screen: Vector2D<i32>,
    facing: Facing,
    walking: bool,
    tick: u32,
    frame: &mut GraphicsFrame,
) {
    // Frames 0-2 face down, 3-5 up, 6-8 right (mirrored for left).
    let base = match facing {
        Facing::Down => 0,
        Facing::Up => 3,
        Facing::Side { .. } => 6,
    };
    let step = if walking {
        1 + (tick as usize / 8) % 2
    } else {
        0
    };
    Object::new(assets::HERO_TOPDOWN.sprite(base + step))
        .set_priority(Priority::P2)
        .set_hflip(matches!(facing, Facing::Side { left: true }))
        .set_pos(screen - vec2(8, 8))
        .show(frame);
}
