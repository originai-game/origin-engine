import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 64,
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
    "line": 1.5,
    "loop": true,
    "name": "Dizzy Stars",
    "outline": "inside",
    "ramps": {
      "main": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ]
    },
    "seed": 74,
    "supersample": 4,
    "w": 112
  }
});
