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
        "count": 12,
        "gain": 0.7,
        "r": 0.9,
        "ramp": "$main",
        "squash": 1,
        "t1": 0.6,
        "type": "rays",
        "width": 0.22,
        "y": 0
      },
      {
        "breakup": 0.6,
        "r0": 0.1,
        "r1": 0.96,
        "ramp": "$main",
        "squash": 1,
        "th": 0.22,
        "type": "ring",
        "y": 0
      },
      {
        "gain": 0.8,
        "r0": 0.05,
        "r1": 0.7,
        "ramp": "$main",
        "squash": 1,
        "t0": 0.18,
        "th": 0.12,
        "type": "ring",
        "y": 0
      },
      {
        "count": 16,
        "drag": 2.4,
        "heat": [
          1,
          0.5
        ],
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 2,
        "speed": [
          2,
          3.2
        ],
        "squash": 1,
        "type": "particles",
        "y": 0
      },
      {
        "dur": 0.1,
        "r": 0.4,
        "ramp": "$main",
        "squash": 1,
        "type": "flash",
        "y": 0
      },
      {
        "count": 8,
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
        "t0": 0.1,
        "type": "particles",
        "x": 0,
        "y": 0
      },
      {
        "count": 5,
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
        "spanX": 0.42,
        "spanY": 0.36,
        "t0": 0.1,
        "type": "particles",
        "x": 0,
        "y": 0
      }
    ],
    "loop": false,
    "name": "Frost Nova",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "frost",
      "main": "ice",
      "smoke": "frost",
      "spark": "frost"
    },
    "seed": 4,
    "w": 64
  }
});
