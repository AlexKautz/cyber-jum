"""Generate all WAV audio assets for Cyber Jum.

Everything is synthesized from scratch in a chiptune style: square-wave
leads, triangle bass, and noise percussion, mirroring the kind of sound the
GBA era was known for.  Music is composed as note lists below, so melodies
are reviewable and editable as plain text.

Output is mono 16-bit PCM at 18157 Hz (one of the GBA mixer rates supported
by the agb crate, so the files can be included without resampling).

Run:  uv run python generate_audio.py
"""

from __future__ import annotations

import wave
from pathlib import Path

import numpy as np

SAMPLE_RATE = 18157
SOUNDS = Path(__file__).resolve().parents[2] / "assets" / "sounds"

# ---------------------------------------------------------------------------
# Synthesis primitives
# ---------------------------------------------------------------------------


def silence(duration: float) -> np.ndarray:
    return np.zeros(int(SAMPLE_RATE * duration))


def square(freq: float, duration: float, duty: float = 0.5,
           vol: float = 1.0) -> np.ndarray:
    t = np.arange(int(SAMPLE_RATE * duration)) / SAMPLE_RATE
    return vol * np.where((t * freq) % 1.0 < duty, 1.0, -1.0)


def triangle(freq: float, duration: float, vol: float = 1.0) -> np.ndarray:
    t = np.arange(int(SAMPLE_RATE * duration)) / SAMPLE_RATE
    return vol * (2.0 * np.abs(2.0 * ((t * freq) % 1.0) - 1.0) - 1.0)


def noise(duration: float, vol: float = 1.0, seed: int = 0) -> np.ndarray:
    rng = np.random.default_rng(seed)
    return vol * rng.uniform(-1.0, 1.0, int(SAMPLE_RATE * duration))


def sweep(f_start: float, f_end: float, duration: float, duty: float = 0.5,
          vol: float = 1.0) -> np.ndarray:
    """Square wave whose pitch glides from f_start to f_end."""
    n = int(SAMPLE_RATE * duration)
    freqs = np.linspace(f_start, f_end, n)
    phase = np.cumsum(freqs) / SAMPLE_RATE
    return vol * np.where(phase % 1.0 < duty, 1.0, -1.0)


def envelope(samples: np.ndarray, attack: float = 0.005,
             release: float = 0.05) -> np.ndarray:
    """Linear attack/release so notes don't click."""
    out = samples.copy()
    a = min(int(SAMPLE_RATE * attack), len(out))
    r = min(int(SAMPLE_RATE * release), len(out))
    if a:
        out[:a] *= np.linspace(0.0, 1.0, a)
    if r:
        out[-r:] *= np.linspace(1.0, 0.0, r)
    return out


def decay(samples: np.ndarray) -> np.ndarray:
    """Exponential fade across the whole sample (percussive feel)."""
    return samples * np.exp(-np.linspace(0.0, 5.0, len(samples)))


# ---------------------------------------------------------------------------
# Notes: name -> frequency.  C4 is middle C.
# ---------------------------------------------------------------------------
NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]


def note_freq(name: str) -> float:
    pitch, octave = name[:-1], int(name[-1])
    semitones = NOTE_NAMES.index(pitch) + (octave + 1) * 12
    return 440.0 * 2.0 ** ((semitones - 69) / 12.0)


Melody = list[tuple[str | None, float]]  # (note name or rest, length in beats)


def render_melody(melody: Melody, bpm: float, wave_fn, vol: float,
                  duty: float | None = None, staccato: float = 0.9) -> np.ndarray:
    """Render a melody track.  Notes are slightly shortened (staccato) and
    padded with silence so they don't bleed together."""
    beat = 60.0 / bpm
    parts = []
    for name, beats in melody:
        dur = beats * beat
        if name is None:
            parts.append(silence(dur))
            continue
        play = dur * staccato
        kwargs = {"duty": duty} if duty is not None else {}
        tone = envelope(wave_fn(note_freq(name), play, vol=vol, **kwargs))
        parts.append(np.concatenate([tone, silence(dur - play)]))
    return np.concatenate(parts)


def drum_track(pattern: str, bpm: float, bars: int) -> np.ndarray:
    """Drums from a per-eighth-note pattern string, repeated each bar.
    'k' = kick, 's' = snare, 'h' = closed hat, '.' = rest."""
    eighth = 60.0 / bpm / 2.0
    hits = {
        "k": decay(sweep(120, 45, eighth, vol=0.9)),
        "s": decay(noise(eighth, vol=0.5, seed=1)),
        "h": decay(noise(eighth, vol=0.18, seed=2))[: int(SAMPLE_RATE * eighth * 0.3)],
        ".": silence(0),
    }
    parts = []
    for _ in range(bars):
        for ch in pattern:
            hit = hits[ch]
            parts.append(np.concatenate([hit, silence(eighth - len(hit) / SAMPLE_RATE)]))
    return np.concatenate(parts)


def mix(*tracks: np.ndarray) -> np.ndarray:
    length = max(len(t) for t in tracks)
    out = np.zeros(length)
    for t in tracks:
        out[: len(t)] += t
    return out


def write_wav(samples: np.ndarray, name: str) -> None:
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.8
    data = (samples * 32767).astype(np.int16)
    SOUNDS.mkdir(parents=True, exist_ok=True)
    path = SOUNDS / name
    with wave.open(str(path), "wb") as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(SAMPLE_RATE)
        f.writeframes(data.tobytes())
    print(f"  {name:24s} {len(samples) / SAMPLE_RATE:5.2f}s")


# ---------------------------------------------------------------------------
# Sound effects
# ---------------------------------------------------------------------------


def sfx() -> None:
    write_wav(envelope(sweep(250, 800, 0.18, duty=0.25)), "jump.wav")
    write_wav(mix(decay(noise(0.10, vol=0.8, seed=3)),
                  decay(sweep(150, 50, 0.10))), "stomp.wav")
    write_wav(decay(sweep(500, 140, 0.30, duty=0.25)), "hurt.wav")
    write_wav(np.concatenate([
        envelope(square(note_freq(n), 0.18, duty=0.25)) for n in ["E4", "C4", "A3"]
    ] + [envelope(square(note_freq("E3"), 0.4, duty=0.25))]), "lose_life.wav")
    write_wav(np.concatenate(
        [envelope(square(note_freq(n), 0.11, duty=0.5)) for n in ["C5", "E5", "G5"]]
        + [envelope(square(note_freq("C6"), 0.45, duty=0.5))]), "win_level.wav")
    write_wav(np.concatenate([
        envelope(square(880, 0.045, duty=0.3)), silence(0.04),
        envelope(square(740, 0.045, duty=0.3)),
    ]), "talk_blip.wav")
    write_wav(envelope(square(660, 0.05, duty=0.5, vol=0.7)), "menu_move.wav")
    write_wav(np.concatenate([
        envelope(square(523, 0.07)), envelope(square(784, 0.12)),
    ]), "menu_select.wav")
    write_wav(np.concatenate([
        envelope(triangle(note_freq(n), 0.16), release=0.1)
        for n in ["C5", "E5", "G5", "C6"]
    ]), "save_chime.wav")


# ---------------------------------------------------------------------------
# Music.  Each song: lead square, harmony square, triangle bass, drums.
# All loops are exact bar multiples so they repeat seamlessly.
# ---------------------------------------------------------------------------


def bass_walk(roots: list[str], beats_per_bar: float = 4.0) -> Melody:
    """Classic root/octave alternating eighth-note bass line."""
    out: Melody = []
    for root in roots:
        up = root[:-1] + str(int(root[-1]) + 1)
        for _ in range(int(beats_per_bar)):
            out += [(root, 0.5), (up, 0.5)]
    return out


def title_theme() -> np.ndarray:
    """Moody synthwave loop in A minor, 110 BPM, 8 bars."""
    bpm = 110
    lead: Melody = [
        ("A4", 1), ("C5", 0.5), ("E5", 0.5), ("A5", 1), ("G5", 0.5), ("E5", 0.5),
        ("F5", 1), ("E5", 0.5), ("C5", 0.5), ("A4", 2),
        ("C5", 1), ("E5", 0.5), ("G5", 0.5), ("E5", 1), ("D5", 0.5), ("C5", 0.5),
        ("B4", 0.5), ("D5", 0.5), ("G5", 1), ("B4", 1), ("G4", 1),
        ("A4", 1), ("C5", 0.5), ("E5", 0.5), ("A5", 1), ("B5", 0.5), ("C6", 0.5),
        ("F5", 1), ("G5", 0.5), ("A5", 0.5), ("F5", 2),
        ("E5", 1), ("C5", 0.5), ("A4", 0.5), ("E5", 1), ("D5", 1),
        ("C5", 0.5), ("B4", 0.5), ("A4", 3),
    ]
    harmony: Melody = [
        ("A3", 4), ("F3", 4), ("C4", 4), ("G3", 4),
        ("A3", 4), ("F3", 4), ("E3", 4), ("E3", 4),
    ]
    bass = bass_walk(["A2", "F2", "C3", "G2", "A2", "F2", "E2", "E2"])
    drums = drum_track("k.h.s.h.", bpm, 8)
    return mix(
        render_melody(lead, bpm, square, vol=0.32, duty=0.25),
        render_melody(harmony, bpm, square, vol=0.10, duty=0.5, staccato=1.0),
        render_melody(bass, bpm, triangle, vol=0.40),
        drums * 0.7,
    )


def overworld_theme() -> np.ndarray:
    """Cheerful walking-around tune in C major, 112 BPM, 8 bars."""
    bpm = 112
    lead: Melody = [
        ("C5", 0.5), ("D5", 0.5), ("E5", 1), ("G5", 1), ("E5", 1),
        ("D5", 0.5), ("E5", 0.5), ("D5", 1), ("B4", 1), ("G4", 1),
        ("A4", 0.5), ("B4", 0.5), ("C5", 1), ("E5", 1), ("C5", 1),
        ("A4", 1), ("G4", 0.5), ("F4", 0.5), ("A4", 2),
        ("C5", 0.5), ("D5", 0.5), ("E5", 1), ("G5", 1), ("A5", 1),
        ("G5", 0.5), ("F5", 0.5), ("E5", 1), ("D5", 1), ("B4", 1),
        ("C5", 0.5), ("E5", 0.5), ("G5", 1), ("E5", 0.5), ("D5", 0.5), ("C5", 1),
        ("D5", 1), ("B4", 1), ("C5", 2),
    ]
    harmony: Melody = [
        ("E4", 4), ("D4", 4), ("C4", 4), ("C4", 4),
        ("E4", 4), ("D4", 4), ("E4", 4), ("G4", 4),
    ]
    bass = bass_walk(["C3", "G2", "A2", "F2", "C3", "G2", "F2", "G2"])
    drums = drum_track("k.h.s.hh", bpm, 8)
    return mix(
        render_melody(lead, bpm, square, vol=0.32, duty=0.5),
        render_melody(harmony, bpm, square, vol=0.08, duty=0.25, staccato=1.0),
        render_melody(bass, bpm, triangle, vol=0.38),
        drums * 0.6,
    )


def platform_theme() -> np.ndarray:
    """Driving challenge-level loop in E minor, 150 BPM, 8 bars."""
    bpm = 150
    arp = lambda a, b, c: [(a, 0.5), (b, 0.5), (c, 0.5), (b, 0.5)] * 2
    lead: Melody = (
        arp("E5", "B4", "G4") + arp("E5", "C5", "G4")
        + arp("F#5", "D5", "A4") + arp("G5", "E5", "B4")
        + [("E5", 0.5), ("G5", 0.5), ("B5", 1), ("A5", 0.5), ("G5", 0.5), ("F#5", 1),
           ("G5", 0.5), ("E5", 0.5), ("D5", 1), ("E5", 0.5), ("B4", 0.5), ("G4", 1),
           ("A4", 0.5), ("C5", 0.5), ("E5", 1), ("D5", 0.5), ("C5", 0.5), ("B4", 1),
           ("E5", 1), ("D5", 0.5), ("B4", 0.5), ("E5", 2)]
    )
    bass = bass_walk(["E2", "C3", "D3", "G2", "E2", "C3", "A2", "B2"])
    drums = drum_track("kkh.s.h.", bpm, 8)
    return mix(
        render_melody(lead, bpm, square, vol=0.34, duty=0.25),
        render_melody(bass, bpm, triangle, vol=0.42),
        drums * 0.7,
    )


def main() -> None:
    print("Generating sound effects...")
    sfx()
    print("Generating music...")
    write_wav(title_theme(), "music_title.wav")
    write_wav(overworld_theme(), "music_overworld.wav")
    write_wav(platform_theme(), "music_platform.wav")
    print("Done.")


if __name__ == "__main__":
    main()
