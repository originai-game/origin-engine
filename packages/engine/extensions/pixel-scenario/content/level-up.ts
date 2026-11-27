import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 24,
    "h": 80,
    "layers": [
      {
        "count": 12,
        "gain": 0.6,
        "r": 1.05,
        "ramp": "$main",
        "t1": 0.8,
        "type": "rays",
        "width": 0.2,
        "y": 0.3
      },
      {
        "gain": 0.7,
        "grow": 0.15,
        "hgt": 2.4,
        "ramp": "$main",
        "speed": 2,
        "type": "pillar",
        "w": 0.5,
        "y": 1.15
      },
      {
        "breakup": 0.4,
        "r0": 0.3,
        "r1": 1,
        "ramp": "$main",
        "squash": 0.32,
        "t1": 0.6,
        "th": 0.16,
        "type": "ring",
        "y": 1.15
      },
      {
        "area": 0.6,
        "areaY": 0.2,
        "count": 7,
        "emit": 0.5,
        "heat": [
          1.05,
          0.6
        ],
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "chevron",
        "size": 2,
        "speed": [
          1,
          1.8
        ],
        "type": "particles",
        "y": 1
      },
      {
        "area": 0.8,
        "areaY": 0.4,
        "count": 8,
        "emit": 0.7,
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
        "y": 0.9
      },
      {
        "gain": 1.05,
        "ramp": "$main",
        "rise": 0.25,
        "t0": 0.12,
        "text": "LEVEL UP",
        "type": "text",
        "y": -0.8
      }
    ],
    "loop": false,
    "name": "Gilded Level Up",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.96
    ],
    "ramps": {
      "alt": "holy",
      "main": "gold",
      "smoke": "white",
      "spark": "gold"
    },
    "seed": 1,
    "w": 64
  }
});
