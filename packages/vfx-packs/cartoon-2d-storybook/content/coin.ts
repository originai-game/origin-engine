import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": false,
    "name": "Coin Grab",
    "outline": "outside",
    "ramps": {
      "main": [
        "#b8700f",
        "#ecaa1e",
        "#ffdc58",
        "#fff6c8"
      ],
      "smoke": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ],
      "spark": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "seed": 61,
    "supersample": 4,
    "w": 96
  }
});
