import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 16,
    "h": 96,
    "layers": [
      {
        "count": 6,
        "flat": 0.15,
        "gain": 0.55,
        "life": [
          0.6,
          1
        ],
        "r0": 0.12,
        "r1": 0.26,
        "ramp": "$smoke",
        "rise": 0.15,
        "spread": 0.55,
        "type": "puffs",
        "x": 0,
        "y": 1.32
      },
      {
        "hgt": 2.7,
        "ramp": "$main",
        "spin": 2,
        "sway": 0.14,
        "twist": 3,
        "type": "tornado",
        "w0": 0.1,
        "w1": 0.78,
        "y": 1.35
      },
      {
        "count": 8,
        "heat": [
          0.8,
          0.8
        ],
        "life": 1,
        "mode": "orbit",
        "orbitR": 0.62,
        "ramp": "$alt",
        "shape": "chunk",
        "size": 2,
        "squash": 0.25,
        "turns": 2,
        "type": "particles",
        "y": 0.4
      },
      {
        "area": 0.48,
        "count": 6,
        "emit": 0,
        "heat": [
          1,
          0.35
        ],
        "life": [
          0.4,
          0.9
        ],
        "mode": "rise",
        "ramp": "$spark",
        "speed": [
          0.6,
          1.3
        ],
        "t0": 0.1,
        "type": "particles",
        "wiggle": 0.07,
        "x": 0,
        "y": 0.5
      }
    ],
    "loop": true,
    "name": "Blaze Tornado",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.9500000000000001
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 2,
    "w": 64
  }
});
