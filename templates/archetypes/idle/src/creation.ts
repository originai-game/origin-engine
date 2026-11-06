// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "idle",
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
  "starter": "idle",
  "seats": 1,
  "controls": {
    "move": "none",
    "aim": "cursor",
    "buttons": [
      {
        "name": "tap",
        "label": "Tap",
        "keys": [
          [
            "Space",
            "Enter"
          ],
          []
        ],
        "pad": [
          "pad.a"
        ],
        "surface": true
      },
      {
        "name": "buy",
        "label": "Buy",
        "keys": [
          [
            "KeyB"
          ],
          []
        ],
        "pad": [
          "pad.x"
        ],
        "surface": true
      },
      {
        "name": "prev",
        "label": "Previous",
        "keys": [
          [
            "ArrowLeft"
          ],
          []
        ],
        "pad": [
          "pad.left"
        ],
        "surface": true
      },
      {
        "name": "next",
        "label": "Next",
        "keys": [
          [
            "ArrowRight"
          ],
          []
        ],
        "pad": [
          "pad.right"
        ],
        "surface": true
      },
      {
        "name": "buy1",
        "label": "Buy Ember",
        "keys": [
          [
            "Digit1"
          ],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "buy2",
        "label": "Buy Storm",
        "keys": [
          [
            "Digit2"
          ],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "buy3",
        "label": "Buy Crystal",
        "keys": [
          [
            "Digit3"
          ],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "buy4",
        "label": "Buy Star",
        "keys": [
          [
            "Digit4"
          ],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "nova",
        "label": "Supernova",
        "keys": [
          [
            "KeyN"
          ],
          []
        ],
        "pad": [
          "pad.y"
        ],
        "surface": true
      },
      {
        "name": "menu",
        "label": "Pause",
        "keys": [
          [
            "KeyP"
          ],
          []
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "point",
        "label": "Tap the orb",
        "keys": [
          [],
          []
        ],
        "pad": [],
        "mouse": "mouse.left",
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
