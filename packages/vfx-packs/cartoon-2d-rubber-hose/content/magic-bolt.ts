import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 10,
    "h": 64,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Magic Bolt",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#725888",
        "#9f7fa9",
        "#ceb1c1",
        "#edd9cb"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 82,
    "supersample": 4,
    "w": 128
  }
});
