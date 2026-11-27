import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 8,
    "h": 48,
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
    "line": 1.5,
    "loop": true,
    "name": "Pellet",
    "outline": "inside",
    "ramps": {
      "main": [
        "#d6785d",
        "#efa86b",
        "#f1d47e",
        "#fbf5da"
      ]
    },
    "seed": 22,
    "supersample": 4,
    "w": 96
  }
});
