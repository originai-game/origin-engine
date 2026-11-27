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
    "line": 3,
    "loop": true,
    "name": "Falling Leaves",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "alt": [
        "#9a603c",
        "#c0884b",
        "#d0ab5f"
      ],
      "main": [
        "#5e783b",
        "#889e4f",
        "#babf6f"
      ]
    },
    "seed": 94,
    "supersample": 4,
    "w": 192
  }
});
