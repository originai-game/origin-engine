import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 20,
    "h": 80,
    "layers": [
      {
        "draw": 0.3,
        "r": 0.95,
        "ramp": "$main",
        "rings": [
          1,
          0.78
        ],
        "runes": 0,
        "squash": 0.32,
        "star": 0,
        "t1": 0.9,
        "ticks": 16,
        "type": "glyph",
        "y": 1.3
      },
      {
        "count": 14,
        "heat": [
          0.6,
          1.05
        ],
        "implodeR": 1.1,
        "life": [
          0.6,
          1
        ],
        "mode": "implode",
        "ramp": "$spark",
        "squash": 1.2,
        "t1": 0.45,
        "type": "particles",
        "y": 0.5
      },
      {
        "grow": 0.1,
        "hgt": 3.2,
        "ramp": "$main",
        "speed": 3,
        "t0": 0.3,
        "type": "pillar",
        "w": 0.28,
        "y": 1.3
      },
      {
        "dur": 0.12,
        "r": 0.7,
        "ramp": "$main",
        "squash": 1.6,
        "t0": 0.32,
        "type": "flash",
        "y": 0.4
      },
      {
        "count": 8,
        "drag": 2.5,
        "heat": [
          1,
          0.6
        ],
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          1,
          2
        ],
        "t0": 0.35,
        "type": "particles",
        "y": 0.4
      }
    ],
    "loop": false,
    "name": "Mystic Teleport",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.89
    ],
    "ramps": {
      "alt": "plasma",
      "main": "arcane",
      "smoke": "dark",
      "spark": "arcane"
    },
    "seed": 1,
    "w": 48
  }
});
