import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 18,
    "h": 96,
    "layers": [
      {
        "count": 7,
        "emit": 0.18,
        "gain": 0.62,
        "life": [
          0.5,
          0.78
        ],
        "r0": 0.16,
        "r1": 0.4,
        "ramp": "$smoke",
        "rise": 0.4,
        "spread": 0.36,
        "t0": 0.22,
        "type": "puffs",
        "y": 0.05
      },
      {
        "cool": 0.62,
        "erode": 1,
        "grow": 0.42,
        "r0": 0.1,
        "r1": 0.74,
        "ramp": "$main",
        "rough": 0.55,
        "t1": 0.86,
        "type": "burst",
        "wobble": 0.32
      },
      {
        "breakup": 0.85,
        "gain": 0.9,
        "r0": 0.25,
        "r1": 0.96,
        "ramp": "$main",
        "t1": 0.55,
        "th": 0.12,
        "thEnd": 0.02,
        "type": "ring"
      },
      {
        "count": 14,
        "drag": 2.2,
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
          1.6,
          2.8
        ],
        "type": "particles"
      },
      {
        "dur": 0.1,
        "r": 0.46,
        "ramp": "$main",
        "type": "flash"
      },
      {
        "area": 0.48,
        "count": 10,
        "emit": 0.6,
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
        "t0": 0.12,
        "type": "particles",
        "wiggle": 0.07,
        "x": 0,
        "y": 0
      }
    ],
    "loop": false,
    "name": "Cinder Explosion",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 7,
    "w": 96
  }
});
