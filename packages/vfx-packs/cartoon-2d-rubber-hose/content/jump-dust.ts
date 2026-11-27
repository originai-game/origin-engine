import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 112,
    "innerLine": 1.5,
    "layers": [
      {
        "cone": 200,
        "count": 6,
        "decay": 0.55,
        "dir": 180,
        "emit": 0.06,
        "flat": 0.2,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.12,
        "r1": 0.26,
        "ramp": "$main",
        "rise": 0.12,
        "speed": 0.9,
        "spread": 0.5,
        "type": "puffs",
        "y": 0.55
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Jump Dust",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.7143
    ],
    "ramps": {
      "main": [
        "#967957",
        "#bc9c72",
        "#dfc597"
      ]
    },
    "scale": 0.571429,
    "seed": 41,
    "supersample": 4,
    "w": 160
  }
});
