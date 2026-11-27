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
        "cone": 40,
        "count": 4,
        "decay": 0.6,
        "dir": 180,
        "flat": 0.4,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.1,
        "r1": 0.2,
        "ramp": "$main",
        "rise": 0.25,
        "speed": 0.8,
        "spread": 0.2,
        "type": "puffs",
        "x": 0.3,
        "y": 0.55
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Run Dust",
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
    "seed": 43,
    "supersample": 4,
    "w": 96
  }
});
