import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 18,
    "h": 64,
    "layers": [
      {
        "count": 6,
        "emit": 0.15,
        "flat": 0.12,
        "gain": 0.55,
        "life": [
          0.5,
          0.8
        ],
        "r0": 0.1,
        "r1": 0.22,
        "ramp": "$smoke",
        "rise": 0.15,
        "spread": 0.85,
        "type": "puffs",
        "y": 0.85
      },
      {
        "count": 5,
        "grow": 0.2,
        "hgt": [
          0.9,
          1.5
        ],
        "hold": 0.6,
        "ramp": "$main",
        "spread": 1.05,
        "type": "spikes",
        "width": [
          0.15,
          0.24
        ],
        "y": 0.88
      },
      {
        "count": 10,
        "dir": -90,
        "drag": 0.8,
        "gravity": 4,
        "heat": [
          0.9,
          0.6
        ],
        "life": [
          0.35,
          0.65
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "chunk",
        "size": [
          2,
          1
        ],
        "speed": [
          1,
          2
        ],
        "spread": 140,
        "t0": 0.02,
        "type": "particles",
        "y": 0.85
      },
      {
        "count": 4,
        "drag": 2.6,
        "heat": [
          1,
          0.55
        ],
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "shard",
        "size": [
          3,
          2
        ],
        "speed": [
          1.4,
          2.4
        ],
        "t0": 0.2,
        "type": "particles",
        "x": 0,
        "y": 0.2
      },
      {
        "count": 3,
        "emit": 0.7,
        "heat": [
          1,
          0.7
        ],
        "life": [
          0.3,
          0.6
        ],
        "mode": "ambient",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "spanX": 0.3733333333333333,
        "spanY": 0.48,
        "t0": 0.2,
        "type": "particles",
        "x": 0,
        "y": 0.2
      }
    ],
    "loop": false,
    "name": "Frozen Spikes",
    "outline": "outside",
    "palette": null,
    "pivot": [
      0.5,
      0.94
    ],
    "ramps": {
      "alt": "frost",
      "main": "ice",
      "smoke": "frost",
      "spark": "frost"
    },
    "seed": 3,
    "w": 96
  }
});
