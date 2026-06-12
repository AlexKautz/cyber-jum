//! The top-down overworld: a dirt road heading north through a forest.
//!
//! The map is a grid of 16x16-pixel cells, written below as ASCII art
//! (one character per cell, row 0 at the top — the hero starts at the
//! bottom and walks up).  Each cell expands to a 2x2 quad of 8x8 tiles
//! from `assets/backgrounds/topdown_tiles.png`.
//!
//! Legend:  T tree   b bush   g grass   f flowers   t tall grass
//!          [ left road edge   . road   ] right road edge

use agb::fixnum::{Vector2D, vec2};

pub const CELL: i32 = 16;
pub const WIDTH_CELLS: i32 = 15;
pub const HEIGHT_CELLS: i32 = 64;
pub const HEIGHT_PX: i32 = HEIGHT_CELLS * CELL;

/// Hero spawn point (overworld pixels), on the road at the bottom of the map.
pub const HERO_START: Vector2D<i32> = vec2(112, 968);

/// Reaching this cell row (only possible once every challenger is beaten)
/// finishes the game.
pub const GOAL_ROW: i32 = 3;

/// Where each challenger stands, in cells: (column, row).  They alternate
/// between the east and west side of the road.  Order matters: index n is
/// challenger n with platformer level n.
pub const NPC_CELLS: [(i32, i32); 5] = [(10, 52), (4, 42), (10, 32), (4, 22), (10, 12)];

#[rustfmt::skip]
pub const MAP: [&str; HEIGHT_CELLS as usize] = [
    "TTTTTTT.TTTTTTT", //  0   forest wall (road vanishes north)
    "TTTTTTT.TTTTTTT", //  1
    "TTTggg[.]gggTTT", //  2
    "TTgggg[.]ggggTT", //  3   <- goal row: the game ends here
    "TTgfgg[.]ggfgTT", //  4
    "Tbgggg[.]ggggbT", //  5
    "TTgggg[.]ggtgTT", //  6
    "TTggtg[.]ggggTT", //  7
    "TTTggg[.]gggTTT", //  8
    "TTgggg[.]ggggTT", //  9
    "TTgfgg[.]ggggTT", // 10
    "Tggggg[.]gggggT", // 11   clearing for the robot (challenger 5)
    "Tggfgg[.]ggggfT", // 12   <- robot at column 10
    "Tggggg[.]gggggT", // 13
    "TTgtgg[.]ggbgTT", // 14
    "TTgggg[.]ggggTT", // 15
    "TbTggg[.]gggTTT", // 16
    "TTgggg[.]ggtgTT", // 17
    "TTgfgg[.]ggggTT", // 18
    "TTgggg[.]ggggTT", // 19
    "TTgggg[.]gggTbT", // 20
    "Tggfgg[.]gggggT", // 21   clearing for the ranger (challenger 4)
    "Tggggg[.]ggfggT", // 22   <- ranger at column 4
    "Tgtggg[.]gggggT", // 23
    "TTgggg[.]ggggTT", // 24
    "TTggbg[.]gtggTT", // 25
    "TTTggg[.]ggggTT", // 26
    "TTgggg[.]gggTTT", // 27
    "TTgtgg[.]ggfgTT", // 28
    "TbTggg[.]ggggTT", // 29
    "TTgggg[.]ggggbT", // 30
    "Tggggg[.]gggggT", // 31   clearing for the woman (challenger 3)
    "Tgfggg[.]ggfggT", // 32   <- woman at column 10
    "Tggggg[.]gggggT", // 33
    "TTggtg[.]ggggTT", // 34
    "TTgggg[.]gtggTT", // 35
    "TTgfgg[.]ggggTT", // 36
    "TTTggg[.]gggTTT", // 37
    "TTgggg[.]ggbgTT", // 38
    "TTggbg[.]ggggTT", // 39
    "TTgggg[.]ggtgTT", // 40
    "Tgggfg[.]gggggT", // 41   clearing for the old man (challenger 2)
    "Tggggg[.]gfgggT", // 42   <- old man at column 4
    "Tgtggg[.]gggggT", // 43
    "TTgggg[.]ggggTT", // 44
    "TTgggg[.]gfggTT", // 45
    "TbTgtg[.]ggggTT", // 46
    "TTgggg[.]gggTbT", // 47
    "TTgfgg[.]gtggTT", // 48
    "TTgggg[.]ggggTT", // 49
    "TTggtg[.]ggggTT", // 50
    "Tggggg[.]gggfgT", // 51   clearing for the kid (challenger 1)
    "Tgfggg[.]gggggT", // 52   <- kid at column 10
    "Tggggg[.]ggtggT", // 53
    "TTgggg[.]ggggTT", // 54
    "TTgtgg[.]gbggTT", // 55
    "TTTggg[.]ggggTT", // 56
    "TTgggg[.]gggTTT", // 57
    "TTgfgg[.]gfggTT", // 58
    "Tbgggg[.]ggggTT", // 59
    "TTgggg[.]ggtgbT", // 60   <- hero starts on the road here
    "TTggtg[.]ggggTT", // 61
    "TTgggg[.]ggggTT", // 62
    "TTTTTT[.]TTTTTT", // 63   forest wall (bottom of the map)
];

/// The cell at (column, row); anything off the map counts as a tree.
pub fn cell(x: i32, y: i32) -> u8 {
    if !(0..WIDTH_CELLS).contains(&x) || !(0..HEIGHT_CELLS).contains(&y) {
        return b'T';
    }
    MAP[y as usize].as_bytes()[x as usize]
}

/// Can the hero walk into this cell?
pub fn walkable(x: i32, y: i32) -> bool {
    !matches!(cell(x, y), b'T' | b'b')
}

// Tile indices into the `topdown_tiles` strip.
const GRASS: usize = 0;
const GRASS_TUFTS: usize = 1;
const GRASS_FLOWER: usize = 2;
const ROAD: usize = 3;
const ROAD_EDGE_LEFT: usize = 4;
const ROAD_EDGE_RIGHT: usize = 5;
const TREE_NW: usize = 6; // the tree's four quads are NW NE / SW SE
const BUSH: usize = 10;
const TALL_GRASS: usize = 11;

/// Map an 8x8 tile coordinate to a tile index in the top-down tile sheet.
/// Each map cell covers a 2x2 quad of tiles.
pub fn tile_at(tile_x: i32, tile_y: i32) -> usize {
    let (quad_x, quad_y) = (tile_x.rem_euclid(2), tile_y.rem_euclid(2));
    match cell(tile_x.div_euclid(2), tile_y.div_euclid(2)) {
        b'T' => TREE_NW + (quad_x + quad_y * 2) as usize,
        b'b' => {
            // A pair of bushes on the diagonal reads as a small thicket.
            if quad_x == quad_y { BUSH } else { GRASS }
        }
        b'f' => {
            if (quad_x, quad_y) == (1, 0) {
                GRASS_FLOWER
            } else {
                GRASS
            }
        }
        b't' => TALL_GRASS,
        b'.' => ROAD,
        b'[' => {
            if quad_x == 0 {
                ROAD_EDGE_LEFT
            } else {
                ROAD
            }
        }
        b']' => {
            if quad_x == 1 {
                ROAD_EDGE_RIGHT
            } else {
                ROAD
            }
        }
        // Plain grass, with deterministic variation so it doesn't tile visibly.
        _ => {
            if (tile_x * 7 + tile_y * 13).rem_euclid(11) == 0 {
                GRASS_TUFTS
            } else {
                GRASS
            }
        }
    }
}
