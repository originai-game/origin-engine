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
        "count": 18,
        "driftX": -0.5,
        "heat": [
          0.95,
          0.75
        ],
        "life": [
          0.85,
          1
        ],
        "mode": "fall",
        "ramp": "nature",
        "shape": "leaf",
        "size": 2,
        "type": "particles",
        "wiggle": 0.12
      }
    ],
    "loop": true,
    "name": "Bloom Ambient",
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
    "seed": 2,
    "w": 128
  }
});
