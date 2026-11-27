import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 176,
    "innerLine": 1.5,
    "layers": [
      {
        "cone": 20,
        "count": 6,
        "decay": 0.4,
        "dir": -90,
        "drift": 0.3,
        "flat": 1,
        "life": [
          0.8,
          1
        ],
        "r0": 0.18,
        "r1": 0.42,
        "ramp": "$main",
        "rise": 1.8,
        "speed": 0.5,
        "spread": 0.15,
        "type": "puffs",
        "y": 1.1
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Chimney Smoke",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.8273
    ],
    "ramps": {
      "main": [
        "#8f806b",
        "#b7a78b",
        "#e0cfad"
      ]
    },
    "seed": 93,
    "supersample": 4,
    "w": 96
  }
});
