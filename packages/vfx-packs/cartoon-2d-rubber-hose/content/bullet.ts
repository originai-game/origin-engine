import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 8,
    "h": 48,
    "innerLine": 1.5,
    "layers": [
      {
        "len": 1.3,
        "r": 0.32,
        "ramp": "$main",
        "rough": 0.2,
        "speed": 2,
        "type": "comet",
        "x": 0.6
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Pellet",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#af6041",
        "#d08b4b",
        "#d5b35b",
        "#f0dea6"
      ]
    },
    "seed": 22,
    "supersample": 4,
    "w": 96
  }
});
