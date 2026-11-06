// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "arena",
  "view": "top-down-2d",
  "look": "16bit-pixel",
  "mood": "bright",
  "ui": "pixel",
  "platform": "desktop",
  "input": "keyboard-mouse",
  "players": "online-lockstep",
  "lang": [
    "en"
  ],
  "music": "ambient",
  "starter": "arena",
  "seats": 2,
  "controls": {
    "move": "stick",
    "aim": "none",
    "buttons": [
      {
        "name": "dash",
        "label": "Dash",
        "keys": [
          [
            "Space",
            "ShiftLeft"
          ],
          [
            "Enter",
            "ShiftRight"
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
