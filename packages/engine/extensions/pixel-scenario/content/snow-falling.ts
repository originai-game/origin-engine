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
        "count": 44,
        "driftX": -0.15,
        "heat": [
          1,
          0.85
        ],
        "life": [
          0.85,
          1
        ],
        "mode": "fall",
        "ramp": "frost",
        "shape": "dot",
        "size": [
          2,
          1
        ],
        "type": "particles",
        "wiggle": 0.08
      }
    ],
    "loop": true,
    "name": "Glacial Ambient",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "frost",
      "main": "ice",
      "smoke": "frost",
      "spark": "frost"
    },
    "seed": 1,
    "w": 128
  }
});
