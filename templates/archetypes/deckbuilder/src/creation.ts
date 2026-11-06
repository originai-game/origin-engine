// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "roguelike-card",
  "view": "side-2d",
  "look": "16bit-pixel",
  "mood": "bright",
  "ui": "pixel",
  "platform": "desktop",
  "input": "keyboard-mouse",
  "players": "single",
  "lang": [
    "en"
  ],
  "music": "ambient",
  "starter": "deckbuilder",
  "seats": 1,
  "controls": {
    "move": "dpad",
    "aim": "none",
    "buttons": [
      {
        "name": "play",
        "label": "Play card",
        "keys": [
          [
            "Space",
            "KeyE"
          ],
          [
            "Enter",
            "Slash"
          ]
        ],
        "pad": [
          "pad.a"
        ],
        "surface": true
      },
      {
        "name": "end",
        "label": "End turn",
        "keys": [
          [
            "KeyR"
          ],
          [
            "Backspace"
          ]
        ],
        "pad": [
          "pad.y"
        ]
      }
    ]
  },
  "camera": {
    "projection": "orthographic"
  },
  "rig": "og.Follow2D",
  "kit": "rpg",
  "quality": "medium",
  "strings": []
} as const;
