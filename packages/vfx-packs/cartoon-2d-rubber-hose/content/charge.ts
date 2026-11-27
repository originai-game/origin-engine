import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 96,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Charge Ball",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#5d7095",
        "#86aeb4",
        "#dae2c8"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 24,
    "supersample": 4,
    "w": 96
  }
});
