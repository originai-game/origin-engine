import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 32,
    "h": 96,
    "layers": [
      {
        "count": 1,
        "dir": -90,
        "drag": 3,
        "life": [
          0.2,
          0.2
        ],
        "mode": "burst",
        "ramp": "$spark",
        "seed": 0,
        "shape": "streak",
        "size": 2,
        "speed": [
          3.2,
          3.2
        ],
        "spread": 6,
        "t0": 0,
        "t1": 0.25,
        "type": "particles",
        "x": -0.55,
        "y": 1
      },
      {
        "count": 34,
        "drag": 2.4,
        "gravity": 0.7,
        "heat": [
          1.15,
          0.5
        ],
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "seed": 10,
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          1.3,
          1.9
        ],
        "t0": 0.18,
        "t1": 0.68,
        "twinkle": 3,
        "type": "particles",
        "x": -0.55,
        "y": -0.6
      },
      {
        "dur": 0.12,
        "r": 0.18,
        "ramp": "$main",
        "t0": 0.18,
        "t1": 0.68,
        "type": "flash",
        "x": -0.55,
        "y": -0.6
      },
      {
        "count": 1,
        "dir": -90,
        "drag": 3,
        "life": [
          0.2,
          0.2
        ],
        "mode": "burst",
        "ramp": "$spark",
        "seed": 1,
        "shape": "streak",
        "size": 2,
        "speed": [
          3.2,
          3.2
        ],
        "spread": 6,
        "t0": 0.25,
        "t1": 0.5,
        "type": "particles",
        "x": 0.5,
        "y": 0.8
      },
      {
        "count": 34,
        "drag": 2.4,
        "gravity": 0.7,
        "heat": [
          1.15,
          0.5
        ],
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$alt",
        "seed": 11,
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          1.3,
          1.9
        ],
        "t0": 0.43,
        "t1": 0.93,
        "twinkle": 3,
        "type": "particles",
        "x": 0.5,
        "y": -0.8
      },
      {
        "dur": 0.12,
        "r": 0.18,
        "ramp": "$main",
        "t0": 0.43,
        "t1": 0.93,
        "type": "flash",
        "x": 0.5,
        "y": -0.8
      },
      {
        "count": 1,
        "dir": -90,
        "drag": 3,
        "life": [
          0.2,
          0.2
        ],
        "mode": "burst",
        "ramp": "$spark",
        "seed": 2,
        "shape": "streak",
        "size": 2,
        "speed": [
          3.2,
          3.2
        ],
        "spread": 6,
        "t0": 0.5,
        "t1": 0.75,
        "type": "particles",
        "x": 0.05,
        "y": 1.35
      },
      {
        "count": 34,
        "drag": 2.4,
        "gravity": 0.7,
        "heat": [
          1.15,
          0.5
        ],
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "seed": 12,
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          1.3,
          1.9
        ],
        "t0": 0.6799999999999999,
        "t1": 1,
        "twinkle": 3,
        "type": "particles",
        "x": 0.05,
        "y": -0.25
      },
      {
        "dur": 0.12,
        "r": 0.18,
        "ramp": "$main",
        "t0": 0.6799999999999999,
        "t1": 1,
        "type": "flash",
        "x": 0.05,
        "y": -0.25
      }
    ],
    "loop": false,
    "name": "Arcane Fireworks",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "plasma",
      "main": "arcane",
      "smoke": "dark",
      "spark": "arcane"
    },
    "seed": 4,
    "w": 96
  }
});
