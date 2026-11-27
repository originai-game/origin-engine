import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
    "innerLine": 1.5,
    "layers": [
      {
        "count": 8,
        "gain": 0.55,
        "r": 0.8,
        "ramp": "$smoke",
        "t1": 0.6,
        "type": "rays",
        "width": 0.16
      },
      {
        "count": 6,
        "drag": 3,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          1.2,
          1.8
        ],
        "t0": 0.1,
        "type": "particles"
      },
      {
        "count": 1,
        "gravity": -2.5,
        "life": [
          0.55,
          0.6
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "coin",
        "size": 11,
        "speed": [
          0,
          0
        ],
        "type": "particles"
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Coin Grab",
    "outline": "outside",
    "outlineColor": "#141010",
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
    "seed": 61,
    "supersample": 4,
    "w": 96
  }
});
