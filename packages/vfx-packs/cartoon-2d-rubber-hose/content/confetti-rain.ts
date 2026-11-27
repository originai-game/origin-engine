import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 192,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 8,
        "driftX": 0.06,
        "fadeIn": 0.15,
        "fallScale": 0.62,
        "life": [
          0.8,
          0.95
        ],
        "mode": "fall",
        "ramp": "$main",
        "shape": "leaf",
        "size": 9,
        "spanX": 0.68,
        "speed": [
          0.6,
          1
        ],
        "type": "particles",
        "wiggle": 0.2,
        "y": 0.3
      },
      {
        "count": 8,
        "driftX": 0.06,
        "fadeIn": 0.15,
        "fallScale": 0.62,
        "life": [
          0.8,
          0.95
        ],
        "mode": "fall",
        "ramp": "$alt",
        "seed": 3,
        "shape": "leaf",
        "size": 9,
        "spanX": 0.68,
        "speed": [
          0.6,
          1
        ],
        "type": "particles",
        "wiggle": 0.2,
        "y": 0.3
      },
      {
        "count": 8,
        "driftX": 0.06,
        "fadeIn": 0.15,
        "fallScale": 0.62,
        "life": [
          0.8,
          0.95
        ],
        "mode": "fall",
        "ramp": "$smoke",
        "seed": 5,
        "shape": "leaf",
        "size": 9,
        "spanX": 0.68,
        "speed": [
          0.6,
          1
        ],
        "type": "particles",
        "wiggle": 0.2,
        "y": 0.3
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Confetti Rain",
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
      ]
    },
    "seed": 68,
    "supersample": 4,
    "w": 192
  }
});
