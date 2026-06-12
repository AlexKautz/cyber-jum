//! The five Cyber Jump challenge levels and their challengers' dialogue.
//!
//! Levels are ASCII art, one character per 16x16 cell, 10 rows tall
//! (exactly one screen) and 60 columns wide (4 screens, scrolling right).
//!
//! Legend:  # solid block          = thin platform (jump up through it)
//!          P hero start           F goal flag (stands on the ground below)
//!          E glitch bug           . empty sky
//!
//! Design rules (enforced by asserts in `platformer.rs`): every P/E/F has
//! ground directly below, pits are at most 3 cells wide and climbs at most
//! 2 cells high, matching the hero's jump (~2.6 cells high, ~3.4 cells far
//! at full speed).

pub struct Level {
    /// The challenger guarding this level.
    pub challenger: &'static str,
    pub map: &'static [&'static str],
    /// Said when the challenger stops the hero on the road.
    pub intro: &'static [&'static str],
    /// Said after the hero beats the level.
    pub outro: &'static [&'static str],
}

pub static LEVELS: [Level; 5] = [
    Level {
        challenger: "Kid",
        #[rustfmt::skip]
        map: &[
            "............................................................",
            "............................................................",
            "............................................................",
            "............................................................",
            "............................................................",
            "............................................................",
            "..........===.........===...................................",
            ".P...........................E.................E.......F....",
            "##########...########...#############...####################",
            "##########...########...#############...####################",
        ],
        intro: &[
            "Kid: Hey! You! You can't go any further until you beat my CYBER JUMP!",
            "Kid: Jump across the gaps and stomp the glitch bugs. Touch one from the side and you're toast!",
            "Kid: Reach my checkered flag to win. You get three tries. Ready? GO!",
        ],
        outro: &[
            "Kid: Whoa, you actually did it!",
            "Kid: Fine, you can pass. But the others up the road are WAY tougher than me!",
        ],
    },
    Level {
        challenger: "Old Man",
        #[rustfmt::skip]
        map: &[
            "............................................................",
            "............................................................",
            "............................................................",
            "..................................E.........................",
            ".................................===........................",
            "................===.........................................",
            "..........................................===...............",
            ".P..........E........E........##...............E.........F..",
            "#######...######...######...##############...########..#####",
            "#######...######...######...##############...########..#####",
        ],
        intro: &[
            "Old Man: In my day, we cyber jumped uphill! Both ways!",
            "Old Man: You can't go any further until you play MY cyber jump, whippersnapper!",
        ],
        outro: &[
            "Old Man: Hmph. Not bad... for a youngster.",
            "Old Man: Go on then. And tell that woman up the road her bugs walk funny!",
        ],
    },
    Level {
        challenger: "Woman",
        #[rustfmt::skip]
        map: &[
            "............................................................",
            "............................................................",
            "............................................................",
            "............................................................",
            "............................E...............................",
            ".....................===...===....E.........................",
            "................===..............===...E...===..............",
            ".P.........===........................===......===....E..F..",
            "#########..........................................#########",
            "#########..........................................#########",
        ],
        intro: &[
            "Woman: Stop right there, traveler!",
            "Woman: My cyber jump has no floor. Only sky. Only platforms. Only GLORY.",
            "Woman: You can't go any further until you cross it!",
        ],
        outro: &[
            "Woman: Graceful! Like a glitch bug in springtime.",
            "Woman: The path is yours. Watch out for the ranger... he's a bit intense.",
        ],
    },
    Level {
        challenger: "Ranger",
        #[rustfmt::skip]
        map: &[
            "............................................................",
            "............................................................",
            "............................................................",
            ".......................E....................................",
            ".......................==...................................",
            "............................................................",
            ".....................#......................................",
            ".P.........E.......E.#....##.E......E........E..........F...",
            "######...######...#############...######...######...########",
            "######...######...#############...######...######...########",
        ],
        intro: &[
            "Ranger: Halt. This road is under my protection.",
            "Ranger: Glitch bugs everywhere. Walls. Pits. My cyber jump is a TRAINING COURSE.",
            "Ranger: You can't go any further until you survive it.",
        ],
        outro: &[
            "Ranger: ...Impressive footwork, recruit.",
            "Ranger: One challenger remains. The robot. Nobody has EVER beaten its cyber jump.",
        ],
    },
    Level {
        challenger: "Robot",
        #[rustfmt::skip]
        map: &[
            "............................................................",
            "............................................................",
            "............................................................",
            "...................................E........................",
            "..................................===.......................",
            "............................................................",
            ".................................#..........................",
            ".P........E......E......E......E.#....##.E......E........F..",
            "#####...#####...####...####...##############...####...######",
            "#####...#####...####...####...##############...####...######",
        ],
        intro: &[
            "Robot: HALT. INTRUDER DETECTED.",
            "Robot: YOU CANNOT GO ANY FURTHER UNTIL YOU COMPLETE... CYBER JUMP PROTOCOL FINAL.",
            "Robot: DIFFICULTY: MAXIMUM. SURVIVAL PROBABILITY: 3.2 PERCENT.",
        ],
        outro: &[
            "Robot: ERROR. ERROR. PROBABILITY MODULE MALFUNCTION.",
            "Robot: ...RECALCULATING. YOU ARE THE CYBER JUMP CHAMPION.",
            "Robot: THE ROAD NORTH IS OPEN. GOODBYE, CHAMPION.",
        ],
    },
];
