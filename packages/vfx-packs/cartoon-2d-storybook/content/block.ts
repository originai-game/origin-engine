import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 144,
    "innerLine": 0.75,
    "layers": [
      {
        "fill": 0,
        "hex": 0,
        "r": 0.62,
        "ramp": "$main",
        "shimmer": 1,
        "th": 0.08,
        "type": "bubble",
        "x": -0.15
      },
      {
        "hollow": 0.9,
        "inner": 0.5,
        "points": 5,
        "pop": 0.25,
        "r": 0.42,
        "ramp": "$spark",
        "t1": 0.45,
        "type": "star",
        "x": 0.55
      },
      {
        "count": 8,
        "dir": 0,
        "drag": 3.5,
        "life": [
          0.3,
          0.55
        ],
        "mode": "spray",
        "ramp": "$spark",
        "shape": "star",
        "size": 7,
        "speed": [
          0.9,
          1.4
        ],
        "spread": 110,
        "type": "particles",
        "x": 0.45
      },
      {
        "dur": 0.1,
        "r": 0.55,
        "ramp": "$spark",
        "squash": 1.6,
        "type": "flash",
        "x": 0.4
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Block",
    "outline": "outside",
    "ramps": {
      "main": [
        "#2856b8",
        "#4a8ee8",
        "#9cd0ff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 16,
    "supersample": 4,
    "w": 128
  }
});
