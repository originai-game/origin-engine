import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 200,
    "innerLine": 0.75,
    "layers": [
      {
        "cool": 0.55,
        "erode": 1,
        "erodeStart": 0.35,
        "gain": 0.9,
        "grow": 0.25,
        "r0": 0.12,
        "r1": 0.5,
        "ramp": "$alt",
        "rough": 0.35,
        "t1": 0.8,
        "type": "burst"
      },
      {
        "hollow": 0.9,
        "inner": 0.36,
        "jitter": 0.3,
        "points": 10,
        "pop": 0.2,
        "r": 0.9,
        "ramp": "$main",
        "sharp": 0.7,
        "type": "star"
      },
      {
        "breakup": 0.6,
        "r0": 0.3,
        "r1": 1,
        "ramp": "$spark",
        "t0": 0.08,
        "t1": 0.75,
        "th": 0.1,
        "type": "ring"
      },
      {
        "count": 10,
        "drag": 2.4,
        "life": [
          0.4,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 7,
        "speed": [
          1.8,
          2.8
        ],
        "t0": 0.05,
        "type": "particles"
      },
      {
        "dur": 0.12,
        "r": 0.55,
        "ramp": "$spark",
        "type": "flash"
      }
    ],
    "line": 1.25,
    "loop": false,
    "name": "Smash Hit",
    "outline": "outside",
    "ramps": {
      "alt": [
        "#d2461e",
        "#ff8a24",
        "#ffd040",
        "#fff6c8"
      ],
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.8,
    "seed": 13,
    "supersample": 4,
    "w": 200
  }
});
