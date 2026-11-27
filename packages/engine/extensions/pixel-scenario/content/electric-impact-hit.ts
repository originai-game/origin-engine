import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 18,
    "frames": 10,
    "h": 48,
    "layers": [
      {
        "breakup": 0.5,
        "r0": 0.2,
        "r1": 0.92,
        "ramp": "$main",
        "t1": 0.65,
        "th": 0.12,
        "thEnd": 0.03,
        "type": "ring"
      },
      {
        "hollow": 0.92,
        "inner": 0.3,
        "jitter": 0.55,
        "points": 7,
        "pop": 0.25,
        "r": 0.9,
        "ramp": "$main",
        "sharp": 1.15,
        "type": "star"
      },
      {
        "count": 8,
        "drag": 3,
        "heat": [
          1,
          0.55
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
          2,
          3.2
        ],
        "type": "particles"
      },
      {
        "dur": 0.15,
        "r": 0.36,
        "ramp": "$main",
        "type": "flash"
      }
    ],
    "loop": false,
    "name": "Static Impact",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "thunder",
      "main": "lightning",
      "smoke": "smoke",
      "spark": "lightning"
    },
    "seed": 3,
    "w": 48
  }
});
