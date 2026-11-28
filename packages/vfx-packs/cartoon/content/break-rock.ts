import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 16,
    "h": 144,
    "innerLine": 1,
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
    "line": 2,
    "loop": false,
    "name": "Rock Break",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#5c5854",
        "#8c8680",
        "#bcb6ae"
      ],
      "smoke": [
        "#8e7f78",
        "#bcaea4",
        "#e9e0d6"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 102,
    "supersample": 4,
    "w": 160
  }
});
