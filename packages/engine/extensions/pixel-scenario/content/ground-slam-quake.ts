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
        "count": 8,
        "emit": 0.15,
        "flat": 0.15,
        "gain": 0.6,
        "life": [
          0.55,
          0.9
        ],
        "r0": 0.12,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.25,
        "spread": 1,
        "type": "puffs",
        "y": 0.65
      },
      {
        "count": 7,
        "grow": 0.18,
        "len": 1.25,
        "ramp": "$main",
        "squash": 0.3,
        "type": "cracks",
        "y": 0.72
      },
      {
        "breakup": 0.6,
        "r0": 0.2,
        "r1": 1.45,
        "ramp": "$main",
        "squash": 0.3,
        "th": 0.26,
        "type": "ring",
        "y": 0.72
      },
      {
        "gain": 0.8,
        "r0": 0.1,
        "r1": 1,
        "ramp": "$main",
        "squash": 0.3,
        "t0": 0.15,
        "th": 0.14,
        "type": "ring",
        "y": 0.72
      },
      {
        "count": 12,
        "dir": -90,
        "drag": 0.6,
        "gravity": 5,
        "heat": [
          0.9,
          0.6
        ],
        "life": [
          0.4,
          0.75
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "chunk",
        "size": [
          3,
          2
        ],
        "speed": [
          1.4,
          2.6
        ],
        "spread": 150,
        "type": "particles",
        "y": 0.68
      },
      {
        "dur": 0.1,
        "r": 0.45,
        "ramp": "$main",
        "squash": 0.6,
        "type": "flash",
        "y": 0.6
      }
    ],
    "loop": false,
    "name": "Stone Ground Slam",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.86
    ],
    "ramps": {
      "alt": "dust",
      "main": "earth",
      "smoke": "dust",
      "spark": "earth"
    },
    "seed": 4,
    "w": 96
  }
});
