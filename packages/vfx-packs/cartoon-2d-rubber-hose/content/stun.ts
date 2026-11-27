import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 64,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 4,
        "life": [
          1,
          1
        ],
        "mode": "orbit",
        "orbitR": 0.75,
        "ramp": "$main",
        "shape": "star",
        "size": 7,
        "squash": 0.35,
        "turns": 1,
        "type": "particles"
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Dizzy Stars",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ]
    },
    "seed": 74,
    "supersample": 4,
    "w": 112
  }
});
