import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 20,
    "h": 184,
    "layers": [
      {
        "count": 10,
        "dir": -90,
        "drag": 1.8,
        "gravity": 1.6,
        "life": [
          0.6,
          1
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "leaf",
        "size": 9,
        "speed": [
          1.8,
          2.8
        ],
        "spread": 120,
        "type": "particles"
      },
      {
        "count": 10,
        "dir": -90,
        "drag": 1.8,
        "gravity": 1.6,
        "life": [
          0.6,
          1
        ],
        "mode": "spray",
        "ramp": "$alt",
        "seed": 3,
        "shape": "leaf",
        "size": 9,
        "speed": [
          1.8,
          2.8
        ],
        "spread": 120,
        "type": "particles"
      },
      {
        "count": 10,
        "dir": -90,
        "drag": 1.8,
        "gravity": 1.6,
        "life": [
          0.6,
          1
        ],
        "mode": "spray",
        "ramp": "$smoke",
        "seed": 5,
        "shape": "leaf",
        "size": 9,
        "speed": [
          1.8,
          2.8
        ],
        "spread": 120,
        "type": "particles"
      },
      {
        "count": 6,
        "drag": 2.6,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          1.4,
          2
        ],
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Confetti Pop",
    "outline": "inside",
    "ramps": {
      "alt": [
        "#5d80cb",
        "#83ace2",
        "#bcdbf8"
      ],
      "main": [
        "#c95d5d",
        "#e08c7b",
        "#f4b89a"
      ],
      "smoke": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "scale": 0.869565,
    "seed": 67,
    "supersample": 4,
    "w": 184
  }
});
