import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 192,
    "innerLine": 0.75,
    "layers": [
      {
        "dur": 0.08,
        "r": 0.25,
        "ramp": "$spark",
        "type": "flash"
      },
      {
        "breakup": 0.7,
        "r0": 0.12,
        "r1": 0.55,
        "ramp": "$spark",
        "t1": 0.4,
        "th": 0.07,
        "type": "ring"
      },
      {
        "count": 14,
        "dir": -90,
        "drag": 1.8,
        "gravity": 3,
        "life": [
          0.5,
          0.9
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "shard",
        "size": [
          7,
          11
        ],
        "speed": [
          1.3,
          2.1
        ],
        "spread": 220,
        "type": "particles"
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Ice Shatter",
    "outline": "outside",
    "ramps": {
      "main": [
        "#3482c8",
        "#7cc6ee",
        "#cdf2ff",
        "#ffffff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.75,
    "seed": 103,
    "supersample": 4,
    "w": 208
  }
});
