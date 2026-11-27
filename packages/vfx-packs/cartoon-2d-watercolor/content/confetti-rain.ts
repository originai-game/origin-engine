import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 192,
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
    "line": 1.5,
    "loop": true,
    "name": "Confetti Rain",
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
      ]
    },
    "seed": 68,
    "supersample": 4,
    "w": 192
  }
});
