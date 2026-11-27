import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 20,
    "h": 184,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": false,
    "name": "Confetti Pop",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "alt": [
        "#526682",
        "#728fa3",
        "#adc1be"
      ],
      "main": [
        "#9c4f44",
        "#c37356",
        "#dd9f72"
      ],
      "smoke": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "scale": 0.869565,
    "seed": 67,
    "supersample": 4,
    "w": 184
  }
});
