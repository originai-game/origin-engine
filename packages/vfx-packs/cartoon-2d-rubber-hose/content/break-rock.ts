import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 16,
    "h": 144,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 6,
        "decay": 0.5,
        "flat": 0.8,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.16,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.25,
        "spread": 0.45,
        "t0": 0.05,
        "t1": 0.85,
        "type": "puffs"
      },
      {
        "hollow": 0.9,
        "inner": 0.45,
        "points": 6,
        "pop": 0.25,
        "r": 0.55,
        "ramp": "$spark",
        "t1": 0.35,
        "type": "star"
      },
      {
        "count": 9,
        "dir": -90,
        "drag": 1.8,
        "gravity": 3.2,
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "chunk",
        "size": [
          7,
          12
        ],
        "speed": [
          1.3,
          2
        ],
        "spread": 200,
        "type": "particles"
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Rock Break",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#665f50",
        "#8f8570",
        "#b9ac91"
      ],
      "smoke": [
        "#8f806b",
        "#b7a78b",
        "#e0cfad"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 102,
    "supersample": 4,
    "w": 160
  }
});
