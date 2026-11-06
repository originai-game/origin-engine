// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "action-platformer",
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
  "starter": "game",
  "seats": 1,
  "controls": {
    "move": "stick",
    "aim": "look",
    "buttons": [
      {
        "name": "jump",
        "label": "Jump",
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
        "name": "run",
        "label": "Run",
        "keys": [
          [
            "ShiftLeft"
          ],
          [
            "ShiftRight"
          ]
        ],
        "pad": [
          "pad.leftStick",
          "pad.rightTrigger"
        ]
      },
      {
        "name": "start",
        "label": "Start",
        "keys": [
          [
            "KeyS",
            "Enter"
          ],
          [
            "Enter"
          ]
        ],
        "pad": [
          "pad.a"
        ],
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
  "kit": null,
  "quality": "medium",
  "strings": []
} as const;
