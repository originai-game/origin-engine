import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 128,
    "layers": [
      {
        "area": 0.4,
        "areaX": 0.3,
        "count": 12,
        "dir": 180,
        "drag": 3,
        "emit": 0.35,
        "life": [
          0.35,
          0.6
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "streak",
        "size": 11,
        "speed": [
          1.6,
          2.4
        ],
        "spread": 12,
        "type": "particles",
        "x": 0.35
      },
      {
        "count": 4,
        "decay": 0.6,
        "dir": 180,
        "life": [
          0.4,
          0.8
        ],
        "r0": 0.12,
        "r1": 0.22,
        "ramp": "$smoke",
        "rise": 0.05,
        "speed": 0.6,
        "spread": 0.25,
        "t0": 0.1,
        "type": "puffs",
        "x": -0.35
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Dash Lines",
    "outline": "inside",
    "ramps": {
      "main": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ],
      "smoke": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "scale": 0.75,
    "seed": 44,
    "supersample": 4,
    "w": 192
  }
});
