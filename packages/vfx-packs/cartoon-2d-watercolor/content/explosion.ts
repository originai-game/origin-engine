import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 18,
    "h": 232,
    "layers": [
      {
        "count": 9,
        "decay": 0.5,
        "emit": 0.12,
        "flat": 0.6,
        "gain": 0.8,
        "life": [
          0.55,
          0.95
        ],
        "r0": 0.18,
        "r1": 0.36,
        "ramp": "$smoke",
        "rise": 0.35,
        "spread": 0.55,
        "t0": 0.18,
        "type": "puffs",
        "y": 0.1
      },
      {
        "cool": 0.5,
        "erode": 0.85,
        "erodeStart": 0.45,
        "grow": 0.35,
        "r0": 0.12,
        "r1": 0.66,
        "ramp": "$main",
        "rough": 0.28,
        "t1": 0.8,
        "type": "burst",
        "y": 0.15
      },
      {
        "breakup": 0.5,
        "r0": 0.2,
        "r1": 0.95,
        "ramp": "$alt",
        "squash": 0.3,
        "t1": 0.55,
        "th": 0.08,
        "type": "ring",
        "y": 0.75
      },
      {
        "count": 12,
        "drag": 2,
        "gravity": 1.2,
        "life": [
          0.4,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 7,
        "speed": [
          1.6,
          2.6
        ],
        "t0": 0.05,
        "type": "particles",
        "y": 0.15
      },
      {
        "dur": 0.1,
        "r": 0.6,
        "ramp": "$spark",
        "type": "flash",
        "y": 0.15
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Boom",
    "outline": "inside",
    "pivot": [
      0.5,
      0.831
    ],
    "ramps": {
      "alt": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ],
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
    "scale": 0.827586,
    "seed": 31,
    "supersample": 4,
    "w": 232
  }
});
