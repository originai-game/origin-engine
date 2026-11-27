import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 200,
    "innerLine": 0.75,
    "layers": [
      {
        "hgt": 2.6,
        "ramp": "$main",
        "rough": 0.3,
        "spin": 2,
        "sway": 0.15,
        "twist": 2,
        "type": "tornado",
        "w0": 0.25,
        "w1": 0.8,
        "y": 1.3
      },
      {
        "cone": 360,
        "count": 9,
        "decay": 0.15,
        "dir": 0,
        "flat": 0.35,
        "life": [
          0.9,
          1
        ],
        "r0": 0.24,
        "r1": 0.32,
        "ramp": "$smoke",
        "rise": 0.04,
        "speed": 0.1,
        "spread": 0.95,
        "type": "puffs",
        "y": -1.2
      },
      {
        "count": 4,
        "life": [
          1,
          1
        ],
        "mode": "orbit",
        "orbitR": 0.6,
        "ramp": "$alt",
        "shape": "leaf",
        "size": 4,
        "squash": 0.3,
        "turns": 2,
        "type": "particles",
        "y": 0.6
      }
    ],
    "line": 1.25,
    "loop": true,
    "name": "Whirlwind",
    "outline": "outside",
    "pivot": [
      0.5,
      0.878
    ],
    "ramps": {
      "alt": [
        "#4f8a26",
        "#86bf3a",
        "#c8e46a"
      ],
      "main": [
        "#3482c8",
        "#7cc6ee",
        "#cdf2ff",
        "#ffffff"
      ],
      "smoke": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.823529,
    "seed": 87,
    "supersample": 4,
    "w": 136
  }
});
