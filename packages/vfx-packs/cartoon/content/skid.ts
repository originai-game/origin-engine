import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 64,
    "innerLine": 1,
    "layers": [
      {
        "cone": 50,
        "count": 5,
        "decay": 0.55,
        "dir": -150,
        "flat": 0.3,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.1,
        "r1": 0.24,
        "ramp": "$main",
        "rise": 0.3,
        "speed": 1,
        "spread": 0.15,
        "type": "puffs",
        "x": 0.4,
        "y": 0.55
      },
      {
        "count": 6,
        "dir": -135,
        "gravity": 2,
        "life": [
          0.3,
          0.5
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "circle",
        "size": 3,
        "speed": [
          0.8,
          1.4
        ],
        "spread": 50,
        "type": "particles",
        "x": 0.45,
        "y": 0.55
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Skid Dust",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "pivot": [
      0.5,
      0.875
    ],
    "ramps": {
      "main": [
        "#a37650",
        "#cfa274",
        "#f0d6ae"
      ]
    },
    "seed": 46,
    "supersample": 4,
    "w": 112
  }
});
