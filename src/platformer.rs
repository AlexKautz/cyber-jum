//! The side-on "Cyber Jump" challenge: run right, stomp glitch bugs, reach
//! the checkered flag.  Touching a bug from the side or falling off the
//! bottom costs a life; three lost lives lose the challenge.
//!
//! Quality-of-life touches (all standard platformer tricks):
//! * coyote time — you can still jump a few frames after leaving a ledge
//! * jump buffering — pressing A just before landing still jumps
//! * variable jump height — release A early for a shorter hop
//! * holding A while stomping a bug gives a higher bounce

use agb::display::object::Object;
use agb::display::tiled::{
    InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, TileFormat, TileSetting,
};
use agb::display::{GraphicsFrame, HEIGHT, Priority, WIDTH};
use agb::fixnum::{Num, Vector2D, num, vec2};
use agb::input::{Button, ButtonController};
use alloc::format;
use alloc::vec::Vec;

use crate::assets;
use crate::audio::Audio;
use crate::levels::LEVELS;
use crate::text::Banner;

pub enum LevelResult {
    Won,
    Lost,
}

type Fix = Num<i32, 8>;
type FixVec = Vector2D<Fix>;

const CELL: i32 = 16;
const ROWS: i32 = 10;
pub const LIVES: u32 = 3;

// Physics, all in pixels per frame (squared).  The level design rules in
// `levels.rs` are derived from these numbers.
const GRAVITY: Fix = num!(0.25);
const JUMP_SPEED: Fix = num!(4.6);
const MAX_FALL: Fix = num!(6.5);
const RUN_ACCEL: Fix = num!(0.2);
const MAX_RUN: Fix = num!(1.5);
const STOMP_BOUNCE: Fix = num!(3.2);
const STOMP_BOUNCE_HELD: Fix = num!(4.8);
const COYOTE_FRAMES: u32 = 6;
const JUMP_BUFFER_FRAMES: u32 = 8;

// Hero collision box (half-extents around the sprite centre).
const HERO_HW: i32 = 5;
const HERO_HH: i32 = 7;
// Glitch bug collision box.
const BUG_HW: i32 = 6;
const BUG_HH: i32 = 5;

// Tile indices in `assets/backgrounds/platform_tiles.png`.
const T_BLOCK_TOP: usize = 0;
const T_BLOCK_FILL: usize = 1;
const T_PLATFORM_L: usize = 2;
const T_PLATFORM_M: usize = 3;
const T_PLATFORM_R: usize = 4;
const T_SKY: usize = 5;
const T_SKY_STARS: usize = 6;
const T_GRID_HILL: usize = 7;
const T_CLOUD_L: usize = 8;
const T_CLOUD_R: usize = 9;
const T_HORIZON: usize = 10;

/// One level's cell grid, with the entity markers stripped out of collision.
struct Map {
    rows: &'static [&'static str],
    width: i32,
}

impl Map {
    fn cell(&self, x: i32, y: i32) -> u8 {
        if !(0..self.width).contains(&x) {
            return b'#'; // the level's sides are solid walls
        }
        if !(0..ROWS).contains(&y) {
            return b'.'; // open sky above, open pit below
        }
        self.rows[y as usize].as_bytes()[x as usize]
    }

    fn is_solid(&self, x: i32, y: i32) -> bool {
        self.cell(x, y) == b'#'
    }

    /// Solid or thin platform: what bugs and falling heroes can stand on.
    fn is_standable(&self, x: i32, y: i32) -> bool {
        matches!(self.cell(x, y), b'#' | b'=')
    }
}

struct Hero {
    pos: FixVec,
    velocity: FixVec,
    facing_left: bool,
    on_ground: bool,
    frames_since_grounded: u32,
    jump_buffer: u32,
}

enum BugState {
    Walking,
    /// Frames left to show the squashed sprite before vanishing.
    Squashed(u32),
}

struct Bug {
    pos: FixVec,
    direction: i32,
    state: BugState,
}

enum Phase {
    /// Showing the "CYBER JUMP n" banner; world is frozen.
    Intro(u32),
    Play,
    /// Hero hit a bug or fell; pause briefly before respawn / game over.
    Dying(u32),
    /// Reached the flag; let the jingle play out.
    Won(u32),
}

pub fn run(
    gfx: &mut agb::display::Graphics,
    audio: &mut Audio,
    input: &mut ButtonController,
    level_index: usize,
) -> LevelResult {
    let level = &LEVELS[level_index];
    let map = Map {
        rows: level.map,
        width: level.map[0].len() as i32,
    };
    validate(&map);

    let (start, flag_cell) = find_markers(&map);
    let mut hero = Hero::new(start);
    let mut bugs = spawn_bugs(&map);
    let mut lives = LIVES;

    let mut backdrop = make_backdrop();
    let mut terrain = InfiniteScrolledMap::new(RegularBackground::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    ));

    let banner = Banner::new(
        &format!(
            "{}'s CYBER JUMP\nChallenge {} of 5",
            level.challenger,
            level_index + 1
        ),
        8,
        11,
    );

    audio.play_music(assets::MUSIC_PLATFORM);

    let mut phase = Phase::Intro(150);
    let mut camera_x;
    let mut tick = 0u32;

    loop {
        input.update();
        tick = tick.wrapping_add(1);

        match phase {
            Phase::Intro(ref mut timer) => {
                *timer -= 1;
                if *timer == 0 || input.is_just_pressed(Button::A) {
                    phase = Phase::Play;
                }
            }
            Phase::Play => {
                hero.update(input, audio, &map);
                update_bugs(&mut bugs, &map, audio);

                if let Some(stomped) = hero.collide_bugs(&mut bugs, input) {
                    if stomped {
                        audio.sfx(assets::SFX_STOMP);
                    } else {
                        audio.sfx(assets::SFX_HURT);
                        phase = Phase::Dying(70);
                    }
                }
                if hero.pos.y.floor() > HEIGHT + CELL {
                    audio.sfx(assets::SFX_HURT);
                    phase = Phase::Dying(70);
                }

                let flag_rect_centre = vec2(flag_cell.x * CELL + 8, flag_cell.y * CELL);
                let delta = hero.pos.floor() - flag_rect_centre;
                if delta.x.abs() < 10 && delta.y.abs() < 24 {
                    audio.stop_music();
                    audio.sfx(assets::SFX_WIN_LEVEL);
                    phase = Phase::Won(130);
                }
            }
            Phase::Dying(ref mut timer) => {
                *timer -= 1;
                if *timer == 40 {
                    audio.sfx(assets::SFX_LOSE_LIFE);
                }
                if *timer == 0 {
                    lives -= 1;
                    if lives == 0 {
                        return LevelResult::Lost;
                    }
                    hero = Hero::new(start);
                    bugs = spawn_bugs(&map);
                    phase = Phase::Play;
                }
            }
            Phase::Won(ref mut timer) => {
                *timer -= 1;
                if *timer == 0 {
                    return LevelResult::Won;
                }
            }
        }

        // Camera: keep the hero centred, clamped to the level.
        let target = hero.pos.x.floor() - WIDTH / 2;
        camera_x = target.clamp(0, map.width * CELL - WIDTH);

        terrain.set_scroll_pos(vec2(camera_x, 0), |pos| terrain_tile(&map, pos));

        // Parallax: the backdrop scrolls at half speed and wraps.
        backdrop.set_scroll_pos(vec2(camera_x / 2, 0));

        let mut frame = gfx.frame();
        backdrop.show(&mut frame);
        terrain.show(&mut frame);

        hero.show(camera_x, tick, matches!(phase, Phase::Dying(_)), &mut frame);
        for bug in &bugs {
            bug.show(camera_x, tick, &mut frame);
        }
        show_flag(flag_cell, camera_x, tick, &mut frame);
        show_hearts(lives, &mut frame);
        if matches!(phase, Phase::Intro(_)) {
            banner.show(&mut frame);
        }

        audio.frame();
        frame.commit();
    }
}

/// Catch level-design mistakes loudly instead of with unwinnable levels.
fn validate(map: &Map) {
    assert_eq!(map.rows.len() as i32, ROWS, "level must be 10 rows tall");
    for row in map.rows {
        assert_eq!(row.len() as i32, map.width, "ragged level row");
    }
    for y in 0..ROWS {
        for x in 0..map.width {
            if matches!(map.cell(x, y), b'P' | b'E' | b'F') {
                assert!(map.is_standable(x, y + 1), "entity floating at {x},{y}");
            }
        }
    }
}

fn find_markers(map: &Map) -> (FixVec, Vector2D<i32>) {
    let mut start = None;
    let mut flag = None;
    for y in 0..ROWS {
        for x in 0..map.width {
            match map.cell(x, y) {
                b'P' => start = Some(vec2(x * CELL + 8, y * CELL + 8).change_base()),
                b'F' => flag = Some(vec2(x, y)),
                _ => {}
            }
        }
    }
    (
        start.expect("level has no P"),
        flag.expect("level has no F"),
    )
}

fn spawn_bugs(map: &Map) -> Vec<Bug> {
    let mut bugs = Vec::new();
    for y in 0..ROWS {
        for x in 0..map.width {
            if map.cell(x, y) == b'E' {
                bugs.push(Bug {
                    // Bugs are squat: their collision centre sits low in the cell.
                    pos: vec2(x * CELL + 8, y * CELL + 9).change_base(),
                    direction: -1,
                    state: BugState::Walking,
                });
            }
        }
    }
    bugs
}

// ---------------------------------------------------------------------------
// Backgrounds
// ---------------------------------------------------------------------------

/// Pick the 8x8 tile for one position of the scrolling terrain layer.
fn terrain_tile(
    map: &Map,
    pos: Vector2D<i32>,
) -> (&'static agb::display::tiled::TileSet, TileSetting) {
    let (cx, cy) = (pos.x.div_euclid(2), pos.y.div_euclid(2));
    let (qx, qy) = (pos.x.rem_euclid(2), pos.y.rem_euclid(2));

    let tile = match map.cell(cx, cy) {
        b'#' => {
            if qy == 0 && map.cell(cx, cy - 1) != b'#' {
                T_BLOCK_TOP
            } else {
                T_BLOCK_FILL
            }
        }
        b'=' if qy == 0 => {
            let left = map.cell(cx - 1, cy) == b'=';
            let right = map.cell(cx + 1, cy) == b'=';
            match (left, right, qx) {
                (false, _, 0) => T_PLATFORM_L,
                (_, false, 1) => T_PLATFORM_R,
                _ => T_PLATFORM_M,
            }
        }
        _ => return (&assets::platform.tiles, TileSetting::BLANK),
    };

    (
        &assets::platform.tiles,
        assets::platform.tile_settings[tile],
    )
}

/// The fixed synthwave backdrop: stars, clouds, horizon glow, grid hills.
fn make_backdrop() -> RegularBackground {
    let mut bg = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    for y in 0..32 {
        for x in 0..32 {
            let tile = match y {
                0..=11 => {
                    // A sparse, deterministic star field.
                    if (x * 13 + y * 7) % 11 == 0 {
                        T_SKY_STARS
                    } else {
                        T_SKY
                    }
                }
                12 => T_HORIZON,
                _ => T_GRID_HILL,
            };
            bg.set_tile(
                vec2(x, y),
                &assets::platform.tiles,
                assets::platform.tile_settings[tile],
            );
        }
    }
    for (cx, cy) in [(3, 3), (11, 5), (19, 2), (26, 6)] {
        for (dx, tile) in [(0, T_CLOUD_L), (1, T_CLOUD_R)] {
            bg.set_tile(
                vec2(cx + dx, cy),
                &assets::platform.tiles,
                assets::platform.tile_settings[tile],
            );
        }
    }
    bg
}

// ---------------------------------------------------------------------------
// Hero
// ---------------------------------------------------------------------------

impl Hero {
    fn new(start: FixVec) -> Self {
        Self {
            pos: start,
            velocity: vec2(num!(0), num!(0)),
            facing_left: false,
            on_ground: false,
            frames_since_grounded: 100,
            jump_buffer: 0,
        }
    }

    fn update(&mut self, input: &ButtonController, audio: &mut Audio, map: &Map) {
        // Run left/right with a little momentum.
        self.velocity.x += RUN_ACCEL * input.x_tri() as i32;
        self.velocity.x = if input.x_tri() as i32 == 0 {
            self.velocity.x * 3 / 4 // friction when no direction is held
        } else {
            self.velocity.x.clamp(-MAX_RUN, MAX_RUN)
        };
        if self.velocity.x < num!(-0.1) {
            self.facing_left = true;
        } else if self.velocity.x > num!(0.1) {
            self.facing_left = false;
        }

        // Jumping, with coyote time and a buffered jump press.
        self.jump_buffer = if input.is_just_pressed(Button::A) {
            JUMP_BUFFER_FRAMES
        } else {
            self.jump_buffer.saturating_sub(1)
        };
        if self.jump_buffer > 0 && self.frames_since_grounded < COYOTE_FRAMES {
            self.velocity.y = -JUMP_SPEED;
            self.jump_buffer = 0;
            self.frames_since_grounded = 100;
            audio.sfx(assets::SFX_JUMP);
        }
        // Variable jump height: releasing A early cuts the rise short.
        if input.is_just_released(Button::A) && self.velocity.y < num!(0) {
            self.velocity.y /= 2;
        }

        self.velocity.y = (self.velocity.y + GRAVITY).clamp(-JUMP_SPEED, MAX_FALL);
        self.move_and_collide(map);

        self.frames_since_grounded = if self.on_ground {
            0
        } else {
            self.frames_since_grounded.saturating_add(1)
        };
    }

    /// Axis-by-axis movement against the cell grid.
    fn move_and_collide(&mut self, map: &Map) {
        // Horizontal.
        let new_x = self.pos.x + self.velocity.x;
        let edge_x = new_x.floor() + self.velocity.x.to_raw().signum() * HERO_HW;
        let cx = edge_x.div_euclid(CELL);
        let collided_x = [-HERO_HH + 2, 0, HERO_HH - 1]
            .iter()
            .any(|dy| map.is_solid(cx, (self.pos.y.floor() + dy).div_euclid(CELL)));
        if collided_x {
            self.velocity.x = num!(0);
        } else {
            self.pos.x = new_x;
        }

        // Vertical.
        let old_feet = self.pos.y.floor() + HERO_HH;
        let new_y = self.pos.y + self.velocity.y;
        self.on_ground = false;

        if self.velocity.y >= num!(0) {
            // Falling: land on solid blocks, and on thin platforms only when
            // crossing their top edge from above (one-way platforms).
            let new_feet = new_y.floor() + HERO_HH;
            let feet_row = new_feet.div_euclid(CELL);
            let columns = [
                (self.pos.x.floor() - HERO_HW + 1).div_euclid(CELL),
                (self.pos.x.floor() + HERO_HW - 1).div_euclid(CELL),
            ];
            let on_solid = columns.iter().any(|&cx| map.is_solid(cx, feet_row));
            let platform_top = feet_row * CELL;
            let on_platform = columns.iter().any(|&cx| map.cell(cx, feet_row) == b'=')
                && old_feet <= platform_top
                && new_feet >= platform_top;

            if (on_solid || on_platform) && new_feet >= feet_row * CELL {
                self.pos.y = Num::new(feet_row * CELL - HERO_HH);
                self.velocity.y = num!(0);
                self.on_ground = true;
            } else {
                self.pos.y = new_y;
            }
        } else {
            // Rising: bump the head on solid blocks only.
            let new_head = new_y.floor() - HERO_HH;
            let head_row = new_head.div_euclid(CELL);
            let columns = [
                (self.pos.x.floor() - HERO_HW + 1).div_euclid(CELL),
                (self.pos.x.floor() + HERO_HW - 1).div_euclid(CELL),
            ];
            if columns.iter().any(|&cx| map.is_solid(cx, head_row)) {
                self.pos.y = Num::new((head_row + 1) * CELL + HERO_HH);
                self.velocity.y = num!(0);
            } else {
                self.pos.y = new_y;
            }
        }
    }

    /// Resolve hero/bug touches.  Returns `Some(true)` for a stomp,
    /// `Some(false)` for a deadly side touch, `None` for no contact.
    fn collide_bugs(&mut self, bugs: &mut [Bug], input: &ButtonController) -> Option<bool> {
        for bug in bugs.iter_mut() {
            if !matches!(bug.state, BugState::Walking) {
                continue;
            }
            let delta = self.pos.floor() - bug.pos.floor();
            if delta.x.abs() >= HERO_HW + BUG_HW || delta.y.abs() >= HERO_HH + BUG_HH {
                continue;
            }
            let falling_onto = self.velocity.y > num!(0.5)
                && (self.pos.y.floor() + HERO_HH) - (bug.pos.y.floor() - BUG_HH) < 8;
            if falling_onto {
                bug.state = BugState::Squashed(40);
                self.velocity.y = if input.is_pressed(Button::A) {
                    -STOMP_BOUNCE_HELD
                } else {
                    -STOMP_BOUNCE
                };
                return Some(true);
            }
            return Some(false);
        }
        None
    }

    fn show(&self, camera_x: i32, tick: u32, dying: bool, frame: &mut GraphicsFrame) {
        let sprite = if dying {
            assets::HERO_PLATFORM.sprite(4)
        } else if !self.on_ground {
            assets::HERO_PLATFORM.sprite(3)
        } else if self.velocity.x.abs() > num!(0.2) {
            assets::HERO_PLATFORM.sprite(1 + (tick as usize / 8) % 2)
        } else {
            assets::HERO_PLATFORM.sprite(0)
        };
        Object::new(sprite)
            .set_priority(Priority::P2)
            .set_hflip(self.facing_left)
            .set_pos(self.pos.floor() - vec2(camera_x + 8, 8))
            .show(frame);
    }
}

// ---------------------------------------------------------------------------
// Bugs and the flag
// ---------------------------------------------------------------------------

fn update_bugs(bugs: &mut [Bug], map: &Map, audio: &mut crate::audio::Audio) {
    for bug in bugs.iter_mut() {
        match bug.state {
            BugState::Squashed(ref mut timer) => {
                if *timer > 0 {
                    *timer -= 1;
                    if *timer == 0 {
                        audio.sfx(crate::assets::SFX_ENEMY_DEATH);
                    }
                }
            }
            BugState::Walking => {
                // Turn around at walls and at the edge of the floor.
                let ahead_x = bug.pos.x.floor() + bug.direction * (BUG_HW + 2);
                let ahead_col = ahead_x.div_euclid(CELL);
                let body_row = bug.pos.y.floor().div_euclid(CELL);
                let feet_row = (bug.pos.y.floor() + BUG_HH + 2).div_euclid(CELL);
                if map.is_solid(ahead_col, body_row) || !map.is_standable(ahead_col, feet_row) {
                    bug.direction = -bug.direction;
                }
                bug.pos.x += Num::new(bug.direction) / 2;
            }
        }
    }
}

impl Bug {
    fn show(&self, camera_x: i32, tick: u32, frame: &mut GraphicsFrame) {
        let screen = self.pos.floor() - vec2(camera_x + 8, 9);
        if !(-32..WIDTH + 32).contains(&screen.x) {
            return;
        }
        let sprite = match self.state {
            BugState::Walking => assets::ENEMY.sprite((tick as usize / 10) % 2),
            BugState::Squashed(0) => return, // gone
            BugState::Squashed(_) => assets::ENEMY.sprite(2),
        };
        Object::new(sprite)
            .set_priority(Priority::P2)
            .set_hflip(self.direction > 0)
            .set_pos(screen)
            .show(frame);
    }
}

fn show_flag(flag_cell: Vector2D<i32>, camera_x: i32, tick: u32, frame: &mut GraphicsFrame) {
    let screen_x = flag_cell.x * CELL - camera_x;
    if !(-32..WIDTH + 32).contains(&screen_x) {
        return;
    }
    // The flag sprite is 16x32; `flag_cell` is where its lower half sits.
    Object::new(assets::FLAG.sprite((tick as usize / 20) % 2))
        .set_priority(Priority::P2)
        .set_pos(vec2(
            screen_x,
            (flag_cell.y - 1) * CELL,
        ))
        .show(frame);
}

fn show_hearts(lives: u32, frame: &mut GraphicsFrame) {
    for i in 0..LIVES {
        let sprite = if i < lives { 0 } else { 1 };
        Object::new(assets::UI.sprite(sprite))
            .set_pos(vec2(4 + i as i32 * 11, 4))
            .show(frame);
    }
}
