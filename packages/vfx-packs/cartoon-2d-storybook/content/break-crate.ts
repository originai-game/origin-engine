import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 16,
    "h": 192,
    "innerLine": 0.75,
    "layers": [
      {
        "count": 5,
        "decay": 0.55,
        "flat": 0.8,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.14,
        "r1": 0.28,
        "ramp": "$smoke",
        "rise": 0.2,
        "spread": 0.4,
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
        "gravity": 3,
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "chunk",
        "size": [
          8,
          12
        ],
        "speed": [
          1.3,
          2.1
        ],
        "spread": 200,
        "type": "particles"
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Crate Break",
    "outline": "outside",
    "ramps": {
      "main": [
        "#6e4024",
        "#a4683a",
        "#d29a5e"
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
    "scale": 0.75,
    "seed": 101,
    "supersample": 4,
    "w": 208
  }
});
