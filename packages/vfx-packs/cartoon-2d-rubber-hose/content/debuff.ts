import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 16,
    "h": 144,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 5,
        "decay": 0.5,
        "emit": 0.4,
        "flat": 0.8,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.14,
        "r1": 0.26,
        "ramp": "$main",
        "rise": -0.2,
        "spread": 0.4,
        "type": "puffs",
        "y": -0.4
      },
      {
        "count": 6,
        "dir": 90,
        "emit": 0.5,
        "gravity": 2.5,
        "life": [
          0.5,
          0.8
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "drop",
        "size": 6,
        "speed": [
          0.4,
          0.8
        ],
        "spread": 50,
        "type": "particles",
        "y": -0.4
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Debuff",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#504253",
        "#776176",
        "#aa949e"
      ]
    },
    "seed": 73,
    "supersample": 4,
    "w": 112
  }
});
