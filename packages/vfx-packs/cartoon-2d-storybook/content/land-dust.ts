import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
    "innerLine": 0.75,
    "layers": [
      {
        "cone": 180,
        "count": 8,
        "decay": 0.5,
        "dir": -90,
        "emit": 0.05,
        "flat": 0.2,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.12,
        "r1": 0.28,
        "ramp": "$main",
        "rise": 0.12,
        "speed": 1.4,
        "spread": 0.9,
        "type": "puffs",
        "y": 0.55
      },
      {
        "count": 6,
        "dir": -90,
        "drag": 1.2,
        "gravity": 2.4,
        "ground": 0.7,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "circle",
        "size": 4,
        "speed": [
          1,
          1.6
        ],
        "spread": 140,
        "type": "particles",
        "y": 0.55
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Land Dust",
    "outline": "outside",
    "pivot": [
      0.5,
      0.7143
    ],
    "ramps": {
      "main": [
        "#a37650",
        "#cfa274",
        "#f0d6ae"
      ]
    },
    "scale": 0.571429,
    "seed": 42,
    "supersample": 4,
    "w": 192
  }
});
