import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 16,
    "h": 192,
    "innerLine": 1.5,
    "layers": [
      {
        "breakup": 0.3,
        "r0": 0.3,
        "r1": 0.9,
        "ramp": "$main",
        "squash": 0.3,
        "t1": 0.6,
        "th": 0.12,
        "type": "ring",
        "y": 1.1
      },
      {
        "area": 0.6,
        "areaY": 0.2,
        "count": 9,
        "emit": 0.5,
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "chevron",
        "size": 10,
        "speed": [
          1.2,
          1.9
        ],
        "type": "particles",
        "y": 1
      },
      {
        "area": 0.7,
        "count": 6,
        "emit": 0.6,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "star",
        "size": 5,
        "speed": [
          0.7,
          1.2
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 1
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Buff",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.8354
    ],
    "ramps": {
      "main": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ],
      "spark": [
        "#9c4f44",
        "#c37356",
        "#dd9f72"
      ]
    },
    "scale": 0.777778,
    "seed": 72,
    "supersample": 4,
    "w": 144
  }
});
