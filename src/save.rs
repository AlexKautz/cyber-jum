//! Saving and loading to the cartridge's battery-backed SRAM.
//!
//! The save stores the hero's overworld position, which challengers have
//! been beaten, and the audio settings.  agb's `SaveSlotManager` handles
//! the SRAM layout, checksums and versioning via the magic header.

use agb::save::SaveSlotManager;
use serde::{Deserialize, Serialize};

/// Changes whenever the save format changes; old saves are then ignored.
const SAVE_MAGIC: [u8; 32] = *b"CYBER-JUM-SAVE-V1_______________";

pub const SLOT: usize = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Hero position in overworld pixels.
    pub hero_x: i32,
    pub hero_y: i32,
    /// Which of the five challengers have been beaten.
    pub beaten: [bool; 5],
    pub music_volume: u8,
    pub sfx_volume: u8,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            hero_x: crate::world_map::HERO_START.x,
            hero_y: crate::world_map::HERO_START.y,
            beaten: [false; 5],
            music_volume: 7,
            sfx_volume: 8,
        }
    }
}

pub struct SaveFile {
    manager: SaveSlotManager,
}

impl SaveFile {
    pub fn new(save: &mut agb::save::SaveManager) -> Self {
        let manager = save
            .init_sram(1, SAVE_MAGIC)
            .expect("failed to initialise SRAM save");
        Self { manager }
    }

    /// The previously saved game, if there is one.
    pub fn load(&mut self) -> Option<SaveData> {
        self.manager.read::<SaveData>(SLOT).ok()
    }

    pub fn store(&mut self, data: &SaveData) {
        // A failed write only costs the player their save, not the game:
        // deliberately ignore the error rather than crash.
        let _ = self.manager.write(SLOT, data, &());
    }
}
