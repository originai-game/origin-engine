import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 16,
    "h": 144,
    "layers": [
      {
        "count": 6,
        "decay": 0.5,
        "flat": 0.8,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.16,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.25,
        "spread": 0.45,
        "t0": 0.05,
        "t1": 0.85,
        "type": "puffs"
      },
      {
        "hollow": 0.9,
        "inner": 0.45,
        "points": 6,
        "pop": 0.25,
        "r": 0.55,
        "ramp": "$spark",
        "t1": 0.35,
        "type": "star"
      },
      {
        "count": 9,
        "dir": -90,
        "drag": 1.8,
        "gravity": 3.2,
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "chunk",
        "size": [
          7,
          12
        ],
        "speed": [
          1.3,
          2
        ],
        "spread": 200,
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Rock Break",
    "outline": "inside",
    "ramps": {
      "main": [
        "#86827d",
        "#a8a4a1",
        "#ccc8c3"
      ],
      "smoke": [
        "#a9a09b",
        "#cbc3bd",
        "#ede8e2"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 102,
    "supersample": 4,
    "w": 160
  }
});
