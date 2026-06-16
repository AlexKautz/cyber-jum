//! Sound playback with user-adjustable music and sound-effect volume.
//!
//! Wraps agb's software mixer.  Music plays on a high-priority looping
//! channel (the mixer never steals it to play an effect); sound effects are
//! fire-and-forget channels.  Volumes go from 0 (mute) to [`MAX_VOLUME`] and
//! are persisted in the save file.

use agb::fixnum::Num;
use agb::sound::mixer::{ChannelId, Mixer, SoundChannel, SoundData};

pub const MAX_VOLUME: u8 = 10;

pub struct Audio<'gba> {
    mixer: Mixer<'gba>,
    music_channel: Option<ChannelId>,
    music_volume: u8,
    sfx_volume: u8,
}

/// Convert a 0-10 user volume into the mixer's fixed-point gain.
fn gain(volume: u8) -> Num<i16, 8> {
    Num::new(volume as i16) / (MAX_VOLUME as i16)
}

impl<'gba> Audio<'gba> {
    pub fn new(mixer: Mixer<'gba>, music_volume: u8, sfx_volume: u8) -> Self {
        Self {
            mixer,
            music_channel: None,
            music_volume,
            sfx_volume,
        }
    }

    /// Must be called once per game frame to keep the mixer's buffers fed.
    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    /// Stop the current music (if any) and start a new looping track.
    pub fn play_music(&mut self, track: SoundData) {
        self.stop_music();
        let mut channel = SoundChannel::new_high_priority(track);
        channel.should_loop().volume(gain(self.music_volume));
        self.music_channel = self.mixer.play_sound(channel);
    }

    pub fn stop_music(&mut self) {
        if let Some(id) = self.music_channel.take()
            && let Some(channel) = self.mixer.channel(&id)
        {
            channel.stop();
        }
    }

    /// Play a one-shot sound effect.
    pub fn sfx(&mut self, effect: SoundData) {
        if self.sfx_volume == 0 {
            return;
        }
        let mut channel = SoundChannel::new(effect);
        channel.volume(gain(self.sfx_volume));
        self.mixer.play_sound(channel);
    }

    pub fn music_volume(&self) -> u8 {
        self.music_volume
    }

    pub fn sfx_volume(&self) -> u8 {
        self.sfx_volume
    }

    /// Change the music volume, applying it immediately to whatever is playing.
    pub fn set_music_volume(&mut self, volume: u8) {
        self.music_volume = volume.min(MAX_VOLUME);
        if let Some(id) = &self.music_channel
            && let Some(channel) = self.mixer.channel(id)
        {
            channel.volume(gain(self.music_volume));
        }
    }

    pub fn set_sfx_volume(&mut self, volume: u8) {
        self.sfx_volume = volume.min(MAX_VOLUME);
    }
}
