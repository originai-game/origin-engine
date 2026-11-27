import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 10,
    "h": 144,
    "innerLine": 1,
    "layers": [
      {
        "hollow": 0.9,
        "inner": 0.42,
        "jitter": 0.25,
        "points": 8,
        "pop": 0.22,
        "r": 0.78,
        "ramp": "$main",
        "sharp": 0.6,
        "type": "star"
      },
      {
        "breakup": 0.5,
        "gain": 0.8,
        "r0": 0.25,
        "r1": 0.95,
        "ramp": "$alt",
        "t0": 0.05,
        "t1": 0.7,
        "th": 0.12,
        "type": "ring"
      },
      {
        "count": 7,
        "drag": 2.6,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 6,
        "speed": [
          1.6,
          2.4
        ],
        "t0": 0.05,
        "type": "particles"
      },
      {
        "dur": 0.12,
        "r": 0.45,
        "ramp": "$spark",
        "type": "flash"
      }
    ],
    "line": 2,
    "loop": false,
    "name": "Pow Hit",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "alt": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ],
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ],
      "spark": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "scale": 0.777778,
    "seed": 11,
    "supersample": 4,
    "w": 144
  }
});
