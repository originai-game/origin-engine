// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "strategy-tower-defense",
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
  "starter": "tower-defense",
  "seats": 1,
  "controls": {
    "move": "none",
    "aim": "cursor",
    "buttons": [
      {
        "name": "place",
        "label": "Plant",
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
        ]
      },
      {
        "name": "tap",
        "label": "Plant here",
        "keys": [
          [],
          []
        ],
        "pad": [],
        "mouse": "mouse.left",
        "surface": true
      },
      {
        "name": "cycle",
        "label": "Next plant",
        "keys": [
          [
            "KeyQ",
            "Tab"
          ],
          [
            "Period"
          ]
        ],
        "pad": [
          "pad.x",
          "pad.rightBumper"
        ]
      },
      {
        "name": "thorn",
        "label": "Thorn",
        "keys": [
          [
            "Digit1"
          ],
          [
            "Numpad1"
          ]
        ],
        "pad": []
      },
      {
        "name": "frost",
        "label": "Frost lily",
        "keys": [
          [
            "Digit2"
          ],
          [
            "Numpad2"
          ]
        ],
        "pad": []
      },
      {
        "name": "hedge",
        "label": "Hedge",
        "keys": [
          [
            "Digit3"
          ],
          [
            "Numpad3"
          ]
        ],
        "pad": []
      },
      {
        "name": "next",
        "label": "Next wave",
        "keys": [
          [
            "KeyN"
          ],
          [
            "Slash"
          ]
        ],
        "pad": [
          "pad.y"
        ]
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
        "pad": []
      },
      {
        "name": "start",
        "label": "Play",
        "keys": [
          [],
          []
        ],
        "pad": []
      },
      {
        "name": "title",
        "label": "To the title",
        "keys": [
          [],
          []
        ],
        "pad": []
      },
      {
        "name": "settings",
        "label": "Settings",
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
      }
    ]
  },
  "camera": {
    "projection": "orthographic"
  },
  "rig": "og.Follow2D",
  "kit": "strategy"
} as const;
