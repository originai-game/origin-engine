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
        "count": 46,
        "driftX": -0.25,
        "fallScale": 1,
        "heat": [
          0.9,
          0.7
        ],
        "life": [
          0.5,
          0.7
        ],
        "mode": "fall",
        "ramp": "water",
        "shape": "streak",
        "size": 3,
        "type": "particles"
      }
    ],
    "loop": true,
    "name": "Aqua Ambient",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "ice",
      "main": "water",
      "smoke": "frost",
      "spark": "water"
    },
    "seed": 1,
    "w": 128
  }
});
