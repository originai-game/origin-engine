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
        "count": 6,
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
        "size": 14,
        "spanX": 0.68,
        "speed": [
          0.4,
          0.7
        ],
        "type": "particles",
        "wiggle": 0.2,
        "y": 0.3
      },
      {
        "count": 5,
        "driftX": 0.06,
        "fadeIn": 0.15,
        "fallScale": 0.62,
        "life": [
          0.8,
          0.95
        ],
        "mode": "fall",
        "ramp": "$alt",
        "seed": 4,
        "shape": "leaf",
        "size": 14,
        "spanX": 0.68,
        "speed": [
          0.4,
          0.7
        ],
        "type": "particles",
        "wiggle": 0.2,
        "y": 0.3
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Falling Leaves",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "alt": [
        "#b4501c",
        "#e88a2a",
        "#f6c04a"
      ],
      "main": [
        "#4f8a26",
        "#86bf3a",
        "#c8e46a"
      ]
    },
    "seed": 94,
    "supersample": 4,
    "w": 192
  }
});
