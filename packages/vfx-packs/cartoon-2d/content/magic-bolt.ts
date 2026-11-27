import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 10,
    "h": 64,
    "innerLine": 1,
    "layers": [
      {
        "len": 1.4,
        "r": 0.36,
        "ramp": "$main",
        "rough": 0.3,
        "speed": 2,
        "type": "comet",
        "x": 0.7
      },
      {
        "area": 0.4,
        "count": 5,
        "life": [
          0.3,
          0.6
        ],
        "mode": "ambient",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "spanX": 1.2,
        "spanY": 0.3,
        "twinkle": 2,
        "type": "particles",
        "x": -0.2
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Magic Bolt",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#5e34c0",
        "#9466ec",
        "#cfb0ff",
        "#f6eeff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 82,
    "supersample": 4,
    "w": 128
  }
});
