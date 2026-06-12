# Testing Cyber Jum in an emulator

## Install mGBA

[mGBA](https://mgba.io) is the most accurate, actively maintained GBA
emulator. Either flavor works:

```sh
brew install mgba          # command-line `mgba` (SDL window)
# or
brew install --cask mgba   # the mGBA.app GUI
```

## Run the game

```sh
mgba cyber_jum.gba
```

or open `cyber_jum.gba` from the mGBA app (File → Load ROM…). During
development you can skip the ROM step entirely — `cargo run --release` is
configured to launch the freshly built game in mGBA.

## Controls

mGBA's default keyboard mapping, with what each button does in the game:

| GBA button | Keyboard | Overworld | Platformer | Menus/dialogue |
|---|---|---|---|---|
| D-pad | Arrow keys | Walk | Run left/right | Move cursor / adjust volume |
| A | X | — | Jump (hold for height; hold while stomping for a big bounce) | Confirm / advance text |
| B | Z | Hold to run | — | Back |
| START | Enter | Pause menu (save, settings, quit) | — | — |

## What to test

1. **Title screen** — music plays; NEW GAME starts; SETTINGS adjusts music
   and sound volume with left/right (changes are audible immediately).
2. **Overworld** — walk up the road (hold B to run); trees and bushes block
   you; grass varies; the camera follows vertically.
3. **Encounter** — near the first challenger a "!" pops up with a blip, they
   walk over and a typewriter dialogue plays (A skips/advances).
4. **Platformer** — banner intro, then: jump across pits, bounce off bug
   heads, die from side contact, fall in a pit to lose a heart, lose all
   three to be sent back to the road's start. Reach the flag to win.
5. **Saving** — pause in the overworld → SAVE GAME → chime + "GAME SAVED!".
   Quit mGBA, relaunch: CONTINUE appears on the title screen and restores
   your spot (and your volume settings). The save lives in `cyber_jum.sav`
   next to the ROM.
6. **Ending** — beat all five challengers and walk off the top of the road.

## Debug logging

mGBA prints agb's panic messages (file + line) to the terminal when launched
as `mgba -l 31 cyber_jum.gba` — useful if an assert in the level data ever
fires.
