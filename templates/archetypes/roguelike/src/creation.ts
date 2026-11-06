// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "roguelike-card",
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
  "starter": "roguelike",
  "seats": 1,
  "controls": {
    "move": "dpad",
    "aim": "cursor",
    "buttons": [
      {
        "name": "wait",
        "label": "Wait",
        "keys": [
          [
            "Space",
            "Period"
          ],
          [
            "Enter",
            "Numpad0"
          ]
        ],
        "pad": [
          "pad.a"
        ]
      },
      {
        "name": "drink",
        "label": "Drink",
        "keys": [
          [
            "KeyQ",
            "Digit1"
          ],
          [
            "Slash",
            "Numpad1"
          ]
        ],
        "pad": [
          "pad.x"
        ]
      },
      {
        "name": "step",
        "label": "Step",
        "keys": [
          [],
          []
        ],
        "pad": [],
        "mouse": "mouse.left",
        "surface": true
      },
      {
        "name": "dive",
        "label": "Dive",
        "keys": [
          [],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "quit",
        "label": "Quit",
        "keys": [
          [],
          []
        ],
        "pad": [],
        "surface": true
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
