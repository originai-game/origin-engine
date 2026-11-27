import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 12,
    "h": 32,
    "layers": [
      {
        "count": 6,
        "heat": [
          1.05,
          0.6
        ],
        "life": [
          0.5,
          1
        ],
        "mode": "ambient",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          3,
          2
        ],
        "spanX": 0.8,
        "spanY": 0.8,
        "twinkle": 1,
        "type": "particles"
      }
    ],
    "loop": true,
    "name": "Sacred Sparkle",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "gold",
      "main": "holy",
      "smoke": "white",
      "spark": "holy"
    },
    "seed": 2,
    "w": 32
  }
});
