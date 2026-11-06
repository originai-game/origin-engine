// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "puzzle-match",
  "view": "top-down-2d",
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
  "starter": "match3",
  "seats": 1,
  "controls": {
    "move": "none",
    "aim": "cursor",
    "buttons": [
      {
        "name": "pick",
        "label": "Pick",
        "keys": [
          [
            "Space"
          ],
          [
            "Enter"
          ]
        ],
        "pad": [
          "pad.a"
        ],
        "mouse": "mouse.left",
        "surface": true
      }
    ]
  },
  "camera": {
    "projection": "orthographic"
  },
  "rig": "og.Follow2D",
  "kit": "puzzle",
  "quality": "medium",
  "strings": []
} as const;
