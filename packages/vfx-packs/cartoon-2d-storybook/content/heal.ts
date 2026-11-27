import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 18,
    "h": 192,
    "innerLine": 0.75,
    "layers": [
      {
        "breakup": 0.25,
        "r0": 0.3,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.32,
        "th": 0.14,
        "type": "ring",
        "y": 1.1
      },
      {
        "area": 0.65,
        "areaY": 0.25,
        "count": 8,
        "emit": 0.5,
        "heat": [
          1.05,
          0.7
        ],
        "life": [
          0.5,
          0.85
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "plus",
        "size": 5,
        "speed": [
          0.8,
          1.5
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
        "ramp": "$spark",
        "shape": "star",
        "size": 3,
        "speed": [
          0.5,
          1
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 1
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Heal",
    "outline": "outside",
    "pivot": [
      0.5,
      0.8354
    ],
    "ramps": {
      "main": [
        "#2a9046",
        "#56cc68",
        "#aef29a",
        "#effff0"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.777778,
    "seed": 71,
    "supersample": 4,
    "w": 144
  }
});
