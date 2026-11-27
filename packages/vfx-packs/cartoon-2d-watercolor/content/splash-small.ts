import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 12,
    "h": 120,
    "layers": [
      {
        "count": 8,
        "dir": -90,
        "drag": 0.6,
        "gravity": 3.2,
        "life": [
          0.45,
          0.8
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "drop",
        "size": 5,
        "speed": [
          1,
          1.8
        ],
        "spread": 90,
        "type": "particles",
        "y": 0.55
      },
      {
        "breakup": 0.2,
        "r0": 0.15,
        "r1": 0.9,
        "ramp": "$alt",
        "squash": 0.28,
        "th": 0.12,
        "type": "ring",
        "y": 0.65
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Plop",
    "outline": "inside",
    "pivot": [
      0.5,
      0.78
    ],
    "ramps": {
      "alt": [
        "#a5d0ea",
        "#d9effb",
        "#f7f7f7"
      ],
      "main": [
        "#6291cd",
        "#7cb5e0",
        "#b3dff7",
        "#f1fafd"
      ]
    },
    "scale": 0.8,
    "seed": 52,
    "supersample": 4,
    "w": 120
  }
});
