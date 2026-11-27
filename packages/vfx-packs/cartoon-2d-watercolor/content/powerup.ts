import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 16,
    "h": 188,
    "layers": [
      {
        "count": 12,
        "gain": 0.7,
        "r": 1,
        "ramp": "$main",
        "spin": 0.2,
        "t1": 0.8,
        "type": "rays",
        "width": 0.18
      },
      {
        "breakup": 0.3,
        "r0": 0.3,
        "r1": 1,
        "ramp": "$main",
        "t1": 0.7,
        "th": 0.14,
        "type": "ring"
      },
      {
        "area": 0.7,
        "count": 10,
        "emit": 0.5,
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          0.8,
          1.5
        ],
        "twinkle": 2,
        "type": "particles"
      }
    ],
    "line": 1.5,
    "loop": false,
    "name": "Power Up",
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
    "scale": 0.820513,
    "seed": 64,
    "supersample": 4,
    "w": 156
  }
});
