import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 120,
    "layers": [
      {
        "count": 6,
        "decay": 0.5,
        "emit": 0.1,
        "flat": 0.7,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.16,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.3,
        "spread": 0.4,
        "t0": 0.2,
        "type": "puffs"
      },
      {
        "cool": 0.5,
        "erode": 0.9,
        "erodeStart": 0.4,
        "grow": 0.3,
        "r0": 0.12,
        "r1": 0.6,
        "ramp": "$main",
        "rough": 0.25,
        "t1": 0.8,
        "type": "burst"
      },
      {
        "count": 7,
        "drag": 2.4,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 6,
        "speed": [
          1.4,
          2.2
        ],
        "t0": 0.05,
        "type": "particles"
      },
      {
        "dur": 0.1,
        "r": 0.5,
        "ramp": "$spark",
        "type": "flash"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Pop Boom",
    "outline": "inside",
    "ramps": {
      "main": [
        "#d1604d",
        "#e39366",
        "#efc172",
        "#f8ebbf"
      ],
      "smoke": [
        "#a9a09b",
        "#cbc3bd",
        "#ede8e2"
      ],
      "spark": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ]
    },
    "seed": 32,
    "supersample": 4,
    "w": 120
  }
});
