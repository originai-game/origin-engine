import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 176,
    "layers": [
      {
        "cone": 20,
        "count": 6,
        "decay": 0.4,
        "dir": -90,
        "drift": 0.3,
        "flat": 1,
        "life": [
          0.8,
          1
        ],
        "r0": 0.18,
        "r1": 0.42,
        "ramp": "$main",
        "rise": 1.8,
        "speed": 0.5,
        "spread": 0.15,
        "type": "puffs",
        "y": 1.1
      }
    ],
    "line": 1.5,
    "loop": true,
    "name": "Chimney Smoke",
    "outline": "inside",
    "pivot": [
      0.5,
      0.8273
    ],
    "ramps": {
      "main": [
        "#a9a09b",
        "#cbc3bd",
        "#ede8e2"
      ]
    },
    "seed": 93,
    "supersample": 4,
    "w": 96
  }
});
