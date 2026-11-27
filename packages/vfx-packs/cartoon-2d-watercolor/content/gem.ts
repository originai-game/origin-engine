import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
    "layers": [
      {
        "breakup": 0.4,
        "r0": 0.2,
        "r1": 0.8,
        "ramp": "$spark",
        "t1": 0.6,
        "th": 0.1,
        "type": "ring"
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
        "shape": "diamond",
        "size": 10,
        "speed": [
          0,
          0
        ],
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Gem Grab",
    "outline": "inside",
    "ramps": {
      "main": [
        "#3ec0bb",
        "#76c7bf",
        "#bbede4"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 62,
    "supersample": 4,
    "w": 96
  }
});
