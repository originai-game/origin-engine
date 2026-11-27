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
        "gain": 0.7,
        "grow": 0.15,
        "hgt": 2.2,
        "ramp": "$main",
        "rough": 0.12,
        "speed": 2,
        "type": "pillar",
        "w": 0.45,
        "y": 1.1
      },
      {
        "breakup": 0.3,
        "r0": 0.25,
        "r1": 0.9,
        "ramp": "$main",
        "squash": 0.32,
        "t1": 0.6,
        "th": 0.14,
        "type": "ring",
        "y": 1.1
      },
      {
        "area": 0.6,
        "count": 8,
        "emit": 0.5,
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "speed": [
          1,
          1.6
        ],
        "type": "particles",
        "y": 1
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Checkpoint",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.8354
    ],
    "ramps": {
      "main": [
        "#427765",
        "#629f84",
        "#abd1ac"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "scale": 0.777778,
    "seed": 70,
    "supersample": 4,
    "w": 144
  }
});
