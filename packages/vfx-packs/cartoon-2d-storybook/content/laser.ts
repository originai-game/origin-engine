import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 8,
    "h": 64,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": true,
    "name": "Ray Beam",
    "outline": "outside",
    "ramps": {
      "main": [
        "#c23a8e",
        "#ee6ac0",
        "#ffb6e6"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 25,
    "supersample": 4,
    "w": 192
  }
});
