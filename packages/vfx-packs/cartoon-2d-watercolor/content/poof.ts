import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 112,
    "layers": [
      {
        "count": 8,
        "decay": 0.45,
        "emit": 0.08,
        "flat": 0.85,
        "life": [
          0.6,
          1
        ],
        "r0": 0.2,
        "r1": 0.4,
        "ramp": "$main",
        "rise": 0.15,
        "spread": 0.35,
        "type": "puffs"
      },
      {
        "count": 5,
        "drag": 2.6,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          1.4,
          2
        ],
        "t0": 0.1,
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Poof",
    "outline": "inside",
    "ramps": {
      "main": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ],
      "spark": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ]
    },
    "seed": 33,
    "supersample": 4,
    "w": 112
  }
});
