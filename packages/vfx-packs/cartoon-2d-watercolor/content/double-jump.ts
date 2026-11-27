import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 80,
    "layers": [
      {
        "breakup": 0.3,
        "r0": 0.2,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.35,
        "th": 0.16,
        "type": "ring",
        "y": 0.4
      },
      {
        "count": 6,
        "drag": 3,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "speed": [
          1.2,
          1.8
        ],
        "squash": 0.4,
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Air Jump",
    "outline": "inside",
    "ramps": {
      "main": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ],
      "spark": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ]
    },
    "seed": 45,
    "supersample": 4,
    "w": 112
  }
});
