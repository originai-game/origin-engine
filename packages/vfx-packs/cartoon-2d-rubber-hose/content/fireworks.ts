import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 22,
    "h": 216,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 1,
        "dir": -90,
        "life": [
          0.25,
          0.25
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 4,
        "speed": [
          2.2,
          2.2
        ],
        "spread": 0,
        "type": "particles",
        "y": 1
      },
      {
        "count": 18,
        "drag": 1.6,
        "gravity": 0.8,
        "life": [
          0.5,
          0.75
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "star",
        "size": 7,
        "speed": [
          1.6,
          2
        ],
        "t0": 0.25,
        "type": "particles",
        "y": -0.1
      },
      {
        "breakup": 0.8,
        "r0": 0.1,
        "r1": 0.6,
        "ramp": "$main",
        "t0": 0.25,
        "t1": 0.5,
        "th": 0.06,
        "type": "ring",
        "y": -0.1
      },
      {
        "dur": 0.08,
        "r": 0.35,
        "ramp": "$spark",
        "t0": 0.25,
        "type": "flash",
        "y": -0.1
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Firework",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#a85d73",
        "#d18297",
        "#ecb5b5"
      ],
      "spark": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ]
    },
    "scale": 0.888889,
    "seed": 69,
    "supersample": 4,
    "w": 216
  }
});
