import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 112,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": false,
    "name": "Sparkle Cast",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#725888",
        "#9f7fa9",
        "#ceb1c1",
        "#edd9cb"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 81,
    "supersample": 4,
    "w": 112
  }
});
