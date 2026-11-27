import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 128,
    "layers": [
      {
        "a0": -150,
        "a1": 40,
        "erode": 0.6,
        "r": 0.72,
        "ramp": "$main",
        "sweep": 0.3,
        "th": 0.24,
        "trail": 0.7,
        "type": "slash"
      },
      {
        "count": 4,
        "drag": 3,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          0.8,
          1.4
        ],
        "t0": 0.18,
        "type": "particles",
        "x": 0.55,
        "y": -0.25
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Swoosh Slash",
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
    "seed": 14,
    "supersample": 4,
    "w": 128
  }
});
