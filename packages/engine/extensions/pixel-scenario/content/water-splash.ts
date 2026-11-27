import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 16,
    "h": 64,
    "layers": [
      {
        "breakup": 0.5,
        "r0": 0.25,
        "r1": 1,
        "ramp": "$main",
        "squash": 0.3,
        "t1": 0.8,
        "th": 0.12,
        "type": "ring",
        "y": 0.78
      },
      {
        "cool": 0.2,
        "erode": 1,
        "grow": 0.3,
        "r0": 0.06,
        "r1": 0.36,
        "ramp": "$main",
        "rough": 0.8,
        "squash": 1.25,
        "t1": 0.55,
        "type": "burst",
        "wobble": 0.45,
        "y": 0.55
      },
      {
        "count": 22,
        "dir": -90,
        "drag": 0.3,
        "gravity": 4.5,
        "ground": 0.82,
        "heat": [
          1,
          0.75
        ],
        "life": [
          0.55,
          0.95
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "dot",
        "size": [
          2,
          1
        ],
        "speed": [
          1.2,
          2.5
        ],
        "spread": 120,
        "type": "particles",
        "y": 0.62
      },
      {
        "count": 6,
        "dir": -90,
        "drag": 0.2,
        "gravity": 5,
        "heat": [
          1.05,
          0.8
        ],
        "life": [
          0.4,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "drop",
        "size": 2,
        "speed": [
          2.2,
          3
        ],
        "spread": 60,
        "type": "particles",
        "y": 0.6
      }
    ],
    "loop": false,
    "name": "Aqua Splash",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.89
    ],
    "ramps": {
      "alt": "ice",
      "main": "water",
      "smoke": "frost",
      "spark": "water"
    },
    "seed": 1,
    "w": 64
  }
});
