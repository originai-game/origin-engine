// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "action-platformer",
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
  "starter": "topdown",
  "seats": 1,
  "controls": {
    "move": "stick",
    "aim": "none",
    "buttons": [
      {
        "name": "attack",
        "label": "Attack",
        "keys": [
          [
            "Space",
            "KeyJ"
          ],
          [
            "Enter",
            "Slash"
          ]
        ],
        "pad": [
          "pad.a"
        ]
      },
      {
        "name": "dash",
        "label": "Dash",
        "keys": [
          [
            "ShiftLeft",
            "KeyK"
          ],
          [
            "ShiftRight",
            "Period"
          ]
        ],
        "pad": [
          "pad.b"
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
