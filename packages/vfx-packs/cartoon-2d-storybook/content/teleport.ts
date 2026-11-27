import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 18,
    "h": 208,
    "innerLine": 0.75,
    "layers": [
      {
        "breakup": 0.2,
        "r0": 0.3,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.32,
        "t1": 0.7,
        "th": 0.14,
        "type": "ring",
        "y": 1.3
      },
      {
        "grow": 0.1,
        "hgt": 2.8,
        "ramp": "$main",
        "rough": 0.12,
        "speed": 3,
        "t0": 0.25,
        "t1": 0.8,
        "type": "pillar",
        "w": 0.4,
        "y": 1.3
      },
      {
        "count": 8,
        "drag": 2.5,
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          1,
          1.8
        ],
        "t0": 0.35,
        "type": "particles",
        "y": 0.3
      },
      {
        "dur": 0.12,
        "r": 0.6,
        "ramp": "$spark",
        "squash": 1.6,
        "t0": 0.3,
        "type": "flash",
        "y": 0.3
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Teleport",
    "outline": "outside",
    "pivot": [
      0.5,
      0.8634
    ],
    "ramps": {
      "main": [
        "#2856b8",
        "#4a8ee8",
        "#9cd0ff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.777778,
    "seed": 85,
    "supersample": 4,
    "w": 144
  }
});
