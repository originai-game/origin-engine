import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 18,
    "h": 160,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 9,
        "gain": 0.55,
        "jitter": 0.2,
        "r": 1.1,
        "ramp": "$smoke",
        "t1": 0.85,
        "type": "rays",
        "width": 0.2,
        "y": 0.2
      },
      {
        "area": 0.5,
        "count": 8,
        "emit": 0.6,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          0.6,
          1.2
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 0.3
      },
      {
        "count": 8,
        "dir": -90,
        "drag": 0.3,
        "emit": 0.2,
        "gravity": 4.5,
        "ground": 0.55,
        "life": [
          0.6,
          0.95
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "coin",
        "size": 8,
        "speed": [
          2,
          2.8
        ],
        "spread": 70,
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Treasure",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.74
    ],
    "ramps": {
      "main": [
        "#9b6f34",
        "#c19645",
        "#dabd68",
        "#f0dea6"
      ],
      "smoke": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ],
      "spark": [
        "#b78b3e",
        "#d4b358",
        "#ead794"
      ]
    },
    "seed": 66,
    "supersample": 4,
    "w": 128
  }
});
