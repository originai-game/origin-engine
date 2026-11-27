import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 16,
    "frames": 14,
    "h": 184,
    "innerLine": 0.75,
    "layers": [
      {
        "grow": 0.3,
        "hgt": 0.9,
        "ramp": "$main",
        "rough": 0.35,
        "speed": 1,
        "t1": 0.6,
        "type": "pillar",
        "w": 0.32,
        "y": 0.7
      },
      {
        "count": 14,
        "dir": -90,
        "drag": 0.6,
        "gravity": 3.2,
        "life": [
          0.5,
          0.9
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "drop",
        "size": 6,
        "speed": [
          1.4,
          2.4
        ],
        "spread": 110,
        "type": "particles",
        "y": 0.6
      },
      {
        "breakup": 0.3,
        "r0": 0.2,
        "r1": 1.08,
        "ramp": "$alt",
        "squash": 0.25,
        "th": 0.1,
        "type": "ring",
        "y": 0.75
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Splash",
    "outline": "outside",
    "pivot": [
      0.5,
      0.813
    ],
    "ramps": {
      "alt": [
        "#7cc4ee",
        "#c6ecff",
        "#ffffff"
      ],
      "main": [
        "#2a6cc0",
        "#3f9fe6",
        "#8fd8ff",
        "#eefaff"
      ]
    },
    "scale": 0.782609,
    "seed": 51,
    "supersample": 4,
    "w": 184
  }
});
