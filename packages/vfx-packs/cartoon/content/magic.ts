import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 112,
    "innerLine": 1,
    "layers": [
      {
        "breakup": 0.4,
        "r0": 0.15,
        "r1": 0.85,
        "ramp": "$main",
        "t1": 0.7,
        "th": 0.14,
        "type": "ring"
      },
      {
        "count": 9,
        "drag": 2.4,
        "life": [
          0.4,
          0.7
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "star",
        "size": 6,
        "speed": [
          1.2,
          2
        ],
        "type": "particles"
      },
      {
        "count": 6,
        "drag": 2.4,
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "circle",
        "size": 3,
        "speed": [
          0.6,
          1.2
        ],
        "t0": 0.1,
        "type": "particles"
      }
    ],
    "line": 2,
    "loop": false,
    "name": "Sparkle Cast",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#5e34c0",
        "#9466ec",
        "#cfb0ff",
        "#f6eeff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 81,
    "supersample": 4,
    "w": 112
  }
});
