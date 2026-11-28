import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 64,
    "innerLine": 1,
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
    "line": 2,
    "loop": true,
    "name": "Dizzy Stars",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "seed": 74,
    "supersample": 4,
    "w": 112
  }
});
