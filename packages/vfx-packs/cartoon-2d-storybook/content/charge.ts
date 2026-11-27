import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 96,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": true,
    "name": "Charge Ball",
    "outline": "outside",
    "ramps": {
      "main": [
        "#2f5ed8",
        "#62bcff",
        "#dff7ff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 24,
    "supersample": 4,
    "w": 96
  }
});
