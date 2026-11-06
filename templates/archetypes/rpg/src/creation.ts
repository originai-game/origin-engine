// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = {
  "genre": "rpg",
  "view": "isometric-2.5d",
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
  "starter": "rpg",
  "seats": 1,
  "controls": {
    "move": "stick",
    "aim": "look",
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
        ],
        "mouse": "mouse.left"
      },
      {
        "name": "talk",
        "label": "Talk",
        "keys": [
          [
            "KeyE"
          ],
          [
            "Period"
          ]
        ],
        "pad": [
          "pad.x"
        ]
      },
      {
        "name": "potion",
        "label": "Potion",
        "keys": [
          [
            "KeyQ"
          ],
          [
            "Comma"
          ]
        ],
        "pad": [
          "pad.y"
        ]
      },
      {
        "name": "choice1",
        "label": "Choice 1",
        "keys": [
          [
            "Digit1"
          ],
          [
            "Numpad1"
          ]
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "choice2",
        "label": "Choice 2",
        "keys": [
          [
            "Digit2"
          ],
          [
            "Numpad2"
          ]
        ],
        "pad": [],
        "surface": true
      },
      {
        "name": "choice3",
        "label": "Choice 3",
        "keys": [
          [
            "Digit3"
          ],
          [
            "Numpad3"
          ]
        ],
        "pad": [],
        "surface": true
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
    "projection": "orthographic",
    "fixedAngle": true,
    "pitch": 35.264,
    "yaw": 45
  },
  "rig": "og.FollowCamera",
  "kit": "rpg",
  "quality": "medium",
  "strings": []
} as const;
