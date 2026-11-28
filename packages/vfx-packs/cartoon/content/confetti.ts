import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 20,
    "h": 184,
    "innerLine": 1,
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
    "line": 2,
    "loop": false,
    "name": "Confetti Pop",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "alt": [
        "#2856b8",
        "#4a8ee8",
        "#9cd0ff"
      ],
      "main": [
        "#b42a2a",
        "#e65a3e",
        "#ff9c6a"
      ],
      "smoke": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.869565,
    "seed": 67,
    "supersample": 4,
    "w": 184
  }
});
