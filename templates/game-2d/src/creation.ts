// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "action-platformer",
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
  "starter": "platformer",
  "seats": 1,
  "controls": {
    "move": "dpad",
    "aim": "none",
    "buttons": [
      {
        "name": "jump",
        "label": "Jump",
        "keys": [
          [
            "Space",
            "KeyW"
          ],
          [
            "Enter",
            "ArrowUp"
          ]
        ],
        "pad": [
          "pad.a"
        ]
      }
    ]
  },
  "camera": {
    "projection": "orthographic"
  },
  "rig": "og.Follow2D",
  "kit": null,
  "quality": "medium",
  "strings": []
} as const;
