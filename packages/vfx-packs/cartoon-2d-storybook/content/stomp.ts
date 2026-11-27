import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 128,
    "innerLine": 0.75,
    "layers": [
      {
        "cone": 180,
        "count": 6,
        "decay": 0.55,
        "dir": 0,
        "emit": 0.08,
        "flat": 0.25,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.14,
        "r1": 0.28,
        "ramp": "$smoke",
        "rise": 0.1,
        "speed": 1.2,
        "spread": 0.9,
        "type": "puffs",
        "y": 0.6
      },
      {
        "hollow": 0.9,
        "inner": 0.45,
        "points": 6,
        "pop": 0.25,
        "r": 0.6,
        "ramp": "$main",
        "type": "star",
        "y": 0.15
      },
      {
        "breakup": 0.4,
        "r0": 0.2,
        "r1": 1.2,
        "ramp": "$alt",
        "squash": 0.3,
        "t1": 0.6,
        "th": 0.12,
        "type": "ring",
        "y": 0.65
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Stomp",
    "outline": "outside",
    "pivot": [
      0.5,
      0.7812
    ],
    "ramps": {
      "alt": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ],
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ],
      "smoke": [
        "#a37650",
        "#cfa274",
        "#f0d6ae"
      ]
    },
    "scale": 0.75,
    "seed": 15,
    "supersample": 4,
    "w": 176
  }
});
