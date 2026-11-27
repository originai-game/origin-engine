import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 8,
    "h": 48,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": true,
    "name": "Pellet",
    "outline": "outside",
    "ramps": {
      "main": [
        "#d2461e",
        "#ff8a24",
        "#ffd040",
        "#fff6c8"
      ]
    },
    "seed": 22,
    "supersample": 4,
    "w": 96
  }
});
