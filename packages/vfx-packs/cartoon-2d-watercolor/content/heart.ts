import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 112,
    "layers": [
      {
        "breakup": 0.3,
        "r0": 0.2,
        "r1": 0.8,
        "ramp": "$main",
        "t1": 0.6,
        "th": 0.1,
        "type": "ring"
      },
      {
        "area": 0.5,
        "count": 4,
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "heart",
        "size": 5,
        "speed": [
          0.8,
          1.4
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
        "shape": "heart",
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
    "name": "Heart Grab",
    "outline": "inside",
    "ramps": {
      "main": [
        "#cd7290",
        "#eb98af",
        "#fad2df"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 63,
    "supersample": 4,
    "w": 96
  }
});
