import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 8,
    "h": 72,
    "layers": [
      {
        "hollow": 0.9,
        "inner": 0.45,
        "points": 5,
        "pop": 0.25,
        "r": 0.7,
        "ramp": "$main",
        "type": "star"
      },
      {
        "count": 5,
        "drag": 3,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "circle",
        "size": 4,
        "speed": [
          1.4,
          2
        ],
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Tap Hit",
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
    "seed": 12,
    "supersample": 4,
    "w": 72
  }
});
