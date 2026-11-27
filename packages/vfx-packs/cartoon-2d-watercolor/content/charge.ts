import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 96,
    "layers": [
      {
        "count": 10,
        "heat": [
          0.6,
          1.05
        ],
        "implodeR": 0.95,
        "life": [
          0.5,
          1
        ],
        "mode": "implode",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "type": "particles"
      },
      {
        "pulse": 0.15,
        "pulseRate": 2,
        "r": 0.34,
        "ramp": "$main",
        "rim": 0.2,
        "spin": 1,
        "type": "orb"
      }
    ],
    "line": 1.5,
    "loop": true,
    "name": "Charge Ball",
    "outline": "inside",
    "ramps": {
      "main": [
        "#708cd5",
        "#95cbf3",
        "#e9f8fd"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 24,
    "supersample": 4,
    "w": 96
  }
});
