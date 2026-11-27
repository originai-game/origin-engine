import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 20,
    "h": 96,
    "layers": [
      {
        "breakup": 0.5,
        "r0": 0.3,
        "r1": 1,
        "ramp": "$main",
        "squash": 0.3,
        "t1": 0.7,
        "th": 0.22,
        "type": "ring",
        "y": 1.72
      },
      {
        "grow": 0.16,
        "hgt": 3.7,
        "ramp": "$main",
        "rough": 0.45,
        "speed": 2,
        "type": "pillar",
        "w": 0.42,
        "y": 1.72
      },
      {
        "area": 0.42,
        "areaY": 0.2,
        "count": 16,
        "emit": 0.55,
        "heat": [
          1,
          0.4
        ],
        "life": [
          0.35,
          0.7
        ],
        "mode": "rise",
        "ramp": "$spark",
        "speed": [
          1.6,
          3
        ],
        "type": "particles",
        "y": 1.6
      },
      {
        "dur": 0.12,
        "r": 0.55,
        "ramp": "$main",
        "squash": 0.6,
        "type": "flash",
        "y": 1.6
      },
      {
        "area": 0.48,
        "count": 8,
        "emit": 0.6,
        "heat": [
          1.05,
          0.6
        ],
        "life": [
          0.45,
          0.9
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          0.4,
          0.9
        ],
        "t0": 0.1,
        "twinkle": 2,
        "type": "particles",
        "x": 0,
        "y": 0.6
      }
    ],
    "loop": false,
    "name": "Divine Pillar",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.93
    ],
    "ramps": {
      "alt": "gold",
      "main": "holy",
      "smoke": "white",
      "spark": "holy"
    },
    "seed": 5,
    "w": 48
  }
});
