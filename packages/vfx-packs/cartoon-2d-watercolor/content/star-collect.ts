import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 16,
    "h": 144,
    "layers": [
      {
        "hollow": 0.6,
        "inner": 0.48,
        "points": 5,
        "pop": 0.3,
        "r": 0.8,
        "ramp": "$main",
        "sharp": 0.2,
        "type": "star"
      },
      {
        "count": 10,
        "gain": 0.7,
        "r": 1,
        "ramp": "$spark",
        "t1": 0.7,
        "type": "rays",
        "width": 0.14
      },
      {
        "count": 10,
        "drag": 2.2,
        "life": [
          0.4,
          0.7
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "star",
        "size": 6,
        "speed": [
          1.4,
          2.2
        ],
        "t0": 0.1,
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Star Grab",
    "outline": "inside",
    "ramps": {
      "main": [
        "#deb059",
        "#f0d57b",
        "#f8f1c4"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 65,
    "supersample": 4,
    "w": 144
  }
});
