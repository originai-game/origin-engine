import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 8,
    "h": 72,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": false,
    "name": "Tap Hit",
    "outline": "outside",
    "ramps": {
      "main": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ],
      "spark": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "seed": 12,
    "supersample": 4,
    "w": 72
  }
});
