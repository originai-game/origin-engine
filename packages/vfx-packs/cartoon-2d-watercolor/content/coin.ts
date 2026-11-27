import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
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
    "line": 1.5,
    "loop": false,
    "name": "Coin Grab",
    "outline": "inside",
    "ramps": {
      "main": [
        "#da963b",
        "#e1ba66",
        "#f2dd8e",
        "#fbf5da"
      ],
      "smoke": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ],
      "spark": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ]
    },
    "seed": 61,
    "supersample": 4,
    "w": 96
  }
});
