// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "shooter",
  "view": "first-person-3d",
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
  "starter": "shooter",
  "seats": 1,
  "controls": {
    "move": "stick",
    "aim": "look",
    "buttons": [
      {
        "name": "fire",
        "label": "Fire",
        "keys": [
          [
            "KeyF"
          ],
          [
            "Slash"
          ]
        ],
        "pad": [
          "pad.rightTrigger"
        ],
        "mouse": "mouse.left"
      },
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
        "name": "reload",
        "label": "Reload",
        "keys": [
          [
            "KeyR"
          ],
          [
            "Period"
          ]
        ],
        "pad": [
          "pad.x"
        ]
      }
    ]
  },
  "camera": {
    "projection": "perspective",
    "fovMin": 60,
    "fovMax": 100
  },
  "rig": "og.FirstPersonCamera",
  "kit": "shooter",
  "quality": "medium",
  "strings": []
} as const;
