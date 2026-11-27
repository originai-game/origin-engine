import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 8,
    "h": 80,
    "innerLine": 1.5,
    "layers": [
      {
        "breakup": 0.6,
        "r0": 0.2,
        "r1": 0.6,
        "ramp": "$main",
        "t1": 0.5,
        "th": 0.14,
        "type": "ring"
      },
      {
        "count": 6,
        "drag": 2.8,
        "life": [
          0.3,
          0.55
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "circle",
        "size": 5,
        "speed": [
          1.4,
          2
        ],
        "type": "particles"
      },
      {
        "dur": 0.12,
        "r": 0.4,
        "ramp": "$spark",
        "type": "flash"
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Pellet Pop",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#af6041",
        "#d08b4b",
        "#d5b35b",
        "#f0dea6"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 23,
    "supersample": 4,
    "w": 80
  }
});
