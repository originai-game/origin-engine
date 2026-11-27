import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 14,
    "h": 176,
    "innerLine": 0.75,
    "layers": [
      {
        "count": 6,
        "fan": 0.6,
        "grow": 0.2,
        "hgt": 1.2,
        "hold": 0.6,
        "lean": 0.35,
        "ramp": "$main",
        "spread": 0.8,
        "type": "spikes",
        "width": 0.22,
        "y": 0.95
      },
      {
        "count": 6,
        "drag": 3,
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "diamond",
        "size": 3,
        "speed": [
          0.8,
          1.4
        ],
        "t0": 0.1,
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Frozen",
    "outline": "outside",
    "pivot": [
      0.5,
      0.8636
    ],
    "ramps": {
      "main": [
        "#3482c8",
        "#7cc6ee",
        "#cdf2ff",
        "#ffffff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.8,
    "seed": 77,
    "supersample": 4,
    "w": 160
  }
});
