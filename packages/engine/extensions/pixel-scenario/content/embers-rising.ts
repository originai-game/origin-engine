import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 24,
    "h": 96,
    "layers": [
      {
        "area": 1,
        "areaX": 1.4,
        "areaY": 0.02,
        "count": 34,
        "heat": [
          1.05,
          0.3
        ],
        "life": [
          0.6,
          1
        ],
        "mode": "rise",
        "ramp": "fire",
        "size": [
          2,
          1
        ],
        "speed": [
          1.6,
          2.6
        ],
        "type": "particles",
        "wiggle": 0.1,
        "y": 1.05
      }
    ],
    "loop": true,
    "name": "Blaze Ambient",
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
    "seed": 2,
    "w": 128
  }
});
