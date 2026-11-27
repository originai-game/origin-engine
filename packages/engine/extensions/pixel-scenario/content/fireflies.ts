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
        "glow": 1,
        "glowFalloff": 0.4,
        "heat": [
          1.1,
          0.6
        ],
        "life": [
          0.7,
          1
        ],
        "mode": "ambient",
        "ramp": "hue:120",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "twinkle": 2,
        "type": "particles",
        "wiggle": 0.14
      }
    ],
    "loop": true,
    "name": "Mystic Ambient",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "plasma",
      "main": "arcane",
      "smoke": "dark",
      "spark": "arcane"
    },
    "seed": 1,
    "w": 128
  }
});
