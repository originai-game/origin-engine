import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 192,
    "innerLine": 1,
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
    "line": 2,
    "loop": true,
    "name": "Confetti Rain",
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
      ]
    },
    "seed": 68,
    "supersample": 4,
    "w": 192
  }
});
