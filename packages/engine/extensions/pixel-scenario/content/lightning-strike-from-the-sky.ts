import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 18,
    "frames": 16,
    "h": 96,
    "layers": [
      {
        "count": 5,
        "emit": 0.15,
        "flat": 0.25,
        "gain": 0.55,
        "life": [
          0.55,
          0.8
        ],
        "r0": 0.12,
        "r1": 0.28,
        "ramp": "$smoke",
        "rise": 0.25,
        "spread": 0.45,
        "t0": 0.2,
        "type": "puffs",
        "x": 0,
        "y": 1.2
      },
      {
        "breakup": 0.6,
        "r0": 0.2,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.32,
        "t0": 0.04,
        "t1": 0.65,
        "th": 0.2,
        "type": "ring",
        "y": 1.25
      },
      {
        "cool": 0.5,
        "erode": 1,
        "grow": 0.4,
        "r0": 0.08,
        "r1": 0.42,
        "ramp": "$main",
        "squash": 0.75,
        "t0": 0.04,
        "t1": 0.7,
        "type": "burst",
        "y": 1.12
      },
      {
        "branch": 0.45,
        "count": 2,
        "depth": 6,
        "fadeStart": 0.3,
        "glow": 1,
        "glowFalloff": 0.5,
        "hold": 1,
        "jag": 0.32,
        "mode": "strike",
        "ramp": "$main",
        "t1": 0.75,
        "thick": 2,
        "type": "bolt",
        "x": 0,
        "y": 1.2
      },
      {
        "count": 12,
        "dir": -90,
        "drag": 1,
        "gravity": 4,
        "heat": [
          1,
          0.5
        ],
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 2,
        "speed": [
          1.6,
          2.8
        ],
        "spread": 130,
        "t0": 0.04,
        "type": "particles",
        "x": 0,
        "y": 1.2
      },
      {
        "dur": 0.14,
        "r": 0.6,
        "ramp": "$main",
        "type": "flash",
        "y": 1.1
      }
    ],
    "loop": false,
    "name": "Storm Strike",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.9166666666666666
    ],
    "ramps": {
      "alt": "wind",
      "main": "lightning",
      "smoke": "smoke",
      "spark": "wind"
    },
    "seed": 4,
    "w": 64
  }
});
