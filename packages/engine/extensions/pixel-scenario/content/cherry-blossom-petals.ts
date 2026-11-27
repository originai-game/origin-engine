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
        "count": 34,
        "driftX": -0.5,
        "heat": [
          1,
          0.8
        ],
        "life": [
          0.85,
          1
        ],
        "mode": "fall",
        "ramp": "hue:350",
        "shape": "petal",
        "size": 2,
        "type": "particles",
        "wiggle": 0.1
      }
    ],
    "loop": true,
    "name": "Thorn Ambient",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "nature",
      "main": "nature",
      "smoke": "nature",
      "spark": "nature"
    },
    "seed": 3,
    "w": 128
  }
});
