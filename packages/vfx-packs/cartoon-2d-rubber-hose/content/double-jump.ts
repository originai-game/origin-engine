import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 80,
    "innerLine": 1.5,
    "layers": [
      {
        "breakup": 0.3,
        "r0": 0.2,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.35,
        "th": 0.16,
        "type": "ring",
        "y": 0.4
      },
      {
        "count": 6,
        "drag": 3,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "speed": [
          1.2,
          1.8
        ],
        "squash": 0.4,
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Air Jump",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ],
      "spark": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ]
    },
    "seed": 45,
    "supersample": 4,
    "w": 112
  }
});
