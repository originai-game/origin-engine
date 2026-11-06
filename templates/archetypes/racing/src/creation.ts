// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "racing",
  "view": "third-person-3d",
  "look": "faceted-low-poly",
  "mood": "bright",
  "ui": "storybook",
  "platform": "desktop",
  "input": "keyboard-mouse",
  "players": "single",
  "lang": [
    "en"
  ],
  "music": "ambient",
  "starter": "racing",
  "seats": 1,
  "controls": {
    "move": "stick",
    "aim": "none",
    "buttons": [
      {
        "name": "throttle",
        "label": "Throttle",
        "keys": [
          [
            "KeyW"
          ],
          [
            "ArrowUp"
          ]
        ],
        "pad": [
          "pad.rightTrigger",
          "pad.a"
        ]
      },
      {
        "name": "brake",
        "label": "Brake",
        "keys": [
          [
            "KeyS"
          ],
          [
            "ArrowDown"
          ]
        ],
        "pad": [
          "pad.leftTrigger",
          "pad.b"
        ]
      },
      {
        "name": "boost",
        "label": "Boost",
        "keys": [
          [
            "Space"
          ],
          [
            "Enter"
          ]
        ],
        "pad": [
          "pad.x"
        ]
      },
      {
        "name": "menu",
        "label": "Menu",
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
    "projection": "perspective",
    "fovMin": 45,
    "fovMax": 75
  },
  "rig": "og.FollowCamera",
  "kit": "racing",
  "quality": "medium",
  "strings": []
} as const;
