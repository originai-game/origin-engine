import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 20,
    "h": 64,
    "layers": [
      {
        "breakup": 0.3,
        "gain": 0.85,
        "r0": 0.3,
        "r1": 0.95,
        "ramp": "$main",
        "squash": 0.35,
        "th": 0.12,
        "type": "ring",
        "y": 1.2
      },
      {
        "dither": 1,
        "gain": 0.45,
        "grow": 0.25,
        "hgt": 2.3,
        "ramp": "$main",
        "rough": 0.3,
        "speed": 1,
        "type": "pillar",
        "w": 0.5,
        "y": 1.2
      },
      {
        "area": 0.65,
        "areaY": 0.25,
        "count": 9,
        "emit": 0.5,
        "heat": [
          1.05,
          0.65
        ],
        "life": [
          0.5,
          0.85
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "cross",
        "size": 1,
        "speed": [
          0.8,
          1.5
        ],
        "type": "particles",
        "y": 1.05
      },
      {
        "area": 0.7,
        "areaY": 0.3,
        "count": 8,
        "emit": 0.6,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          0.5,
          1
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 1
      }
    ],
    "loop": false,
    "name": "Bloom Heal",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.95
    ],
    "ramps": {
      "alt": "nature",
      "main": "nature",
      "smoke": "nature",
      "spark": "nature"
    },
    "seed": 2,
    "w": 48
  }
});
