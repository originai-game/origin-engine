import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 18,
    "frames": 12,
    "h": 64,
    "layers": [
      {
        "angle": 58,
        "count": 3,
        "curve": 0.2,
        "gap": 0.24,
        "len": 1.45,
        "mode": "claw",
        "ramp": "$main",
        "th": 2.4,
        "type": "streaks"
      },
      {
        "count": 6,
        "dir": 58,
        "drag": 3,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 2,
        "speed": [
          1.2,
          2
        ],
        "spread": 80,
        "t0": 0.1,
        "type": "particles"
      },
      {
        "count": 7,
        "drag": 1,
        "gravity": 4,
        "heat": [
          0.95,
          0.55
        ],
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "dot",
        "size": [
          2,
          1
        ],
        "speed": [
          1.4,
          2.8
        ],
        "t0": 0.1,
        "type": "particles",
        "x": 0,
        "y": 0
      }
    ],
    "loop": false,
    "name": "Sanguine Claw",
    "outline": "outside",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "blood",
      "main": "blood",
      "smoke": "blood",
      "spark": "blood"
    },
    "seed": 1,
    "w": 64
  }
});
