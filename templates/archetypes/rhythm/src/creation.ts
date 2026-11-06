// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "rhythm",
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
  "starter": "rhythm",
  "seats": 1,
  "controls": {
    "move": "none",
    "aim": "none",
    "buttons": [
      {
        "name": "lane0",
        "label": "Left lane",
        "keys": [
          [
            "KeyA"
          ],
          [
            "KeyJ",
            "ArrowLeft"
          ]
        ],
        "pad": [
          "pad.left",
          "pad.x"
        ]
      },
      {
        "name": "lane1",
        "label": "Middle lane",
        "keys": [
          [
            "KeyS",
            "Space"
          ],
          [
            "KeyK",
            "ArrowDown"
          ]
        ],
        "pad": [
          "pad.down",
          "pad.a"
        ]
      },
      {
        "name": "lane2",
        "label": "Right lane",
        "keys": [
          [
            "KeyD"
          ],
          [
            "KeyL",
            "ArrowRight"
          ]
        ],
        "pad": [
          "pad.right",
          "pad.b"
        ]
      },
      {
        "name": "start",
        "label": "Start",
        "keys": [
          [],
          []
        ],
        "pad": []
      },
      {
        "name": "resume",
        "label": "Resume",
        "keys": [
          [],
          []
        ],
        "pad": []
      },
      {
        "name": "title",
        "label": "Title",
        "keys": [
          [],
          []
        ],
        "pad": []
      },
      {
        "name": "calibrate",
        "label": "Calibrate",
        "keys": [
          [],
          []
        ],
        "pad": []
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
