import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 20,
    "frames": 6,
    "h": 64,
    "impact": 0,
    "layers": [
      {
        "hollow": 0.8,
        "inner": 0.42,
        "points": 6,
        "pop": 0.3,
        "r": 0.62,
        "ramp": "$main",
        "type": "star",
        "x": -0.2
      },
      {
        "cool": 0.6,
        "grow": 0.2,
        "r0": 0.1,
        "r1": 0.34,
        "ramp": "$main",
        "rough": 0.2,
        "t1": 0.7,
        "type": "burst",
        "x": -0.1
      },
      {
        "count": 3,
        "decay": 0.5,
        "dir": 0,
        "emit": 0.2,
        "life": [
          0.5,
          0.8
        ],
        "r0": 0.08,
        "r1": 0.18,
        "ramp": "$smoke",
        "rise": 0.15,
        "speed": 0.6,
        "spread": 0.15,
        "t0": 0.3,
        "type": "puffs",
        "x": 0.1
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Pop Muzzle",
    "outline": "inside",
    "ramps": {
      "main": [
        "#d6785d",
        "#efa86b",
        "#f1d47e",
        "#fbf5da"
      ],
      "smoke": [
        "#a9a09b",
        "#cbc3bd",
        "#ede8e2"
      ]
    },
    "seed": 21,
    "supersample": 4,
    "w": 96
  }
});
