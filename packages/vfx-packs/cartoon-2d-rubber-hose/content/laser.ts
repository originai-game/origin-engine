import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 8,
    "h": 64,
    "innerLine": 1.5,
    "layers": [
      {
        "cap": 0.3,
        "grow": 0,
        "len": 2.9,
        "ramp": "$main",
        "speed": 2,
        "th": 0.28,
        "type": "beam",
        "wob": 0.12,
        "x": -1.45
      },
      {
        "area": 0.9,
        "count": 6,
        "life": [
          0.3,
          0.6
        ],
        "mode": "ambient",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "spanX": 0.85,
        "spanY": 0.25,
        "speed": [
          0,
          0.1
        ],
        "twinkle": 2,
        "type": "particles",
        "wiggle": 0.04
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Ray Beam",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#a85d73",
        "#d18297",
        "#ecb5b5"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 25,
    "supersample": 4,
    "w": 192
  }
});
