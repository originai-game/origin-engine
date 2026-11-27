import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 8,
    "h": 80,
    "innerLine": 1,
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
    "line": 2,
    "loop": false,
    "name": "Pellet Pop",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#d2461e",
        "#ff8a24",
        "#ffd040",
        "#fff6c8"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 23,
    "supersample": 4,
    "w": 80
  }
});
