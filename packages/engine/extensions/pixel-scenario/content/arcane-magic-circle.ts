import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 24,
    "h": 48,
    "layers": [
      {
        "dither": 0.8,
        "fill": 0.14,
        "glow": 1,
        "glowFalloff": 0.5,
        "r": 1.62,
        "ramp": "$main",
        "rings": [
          1,
          0.84,
          0.55
        ],
        "runes": 10,
        "spin": 1,
        "squash": 0.42,
        "star": 6,
        "step": 2,
        "ticks": 32,
        "type": "glyph",
        "y": 0.2
      },
      {
        "area": 1.4,
        "areaY": 0.3,
        "count": 10,
        "heat": [
          1,
          0.4
        ],
        "life": [
          0.5,
          0.9
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "dot",
        "speed": [
          0.5,
          1.1
        ],
        "type": "particles",
        "y": 0.2
      },
      {
        "count": 4,
        "drag": 2,
        "emit": 0,
        "heat": [
          1.05,
          0.6
        ],
        "life": [
          0.4,
          0.8
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          0.6,
          1.4
        ],
        "t0": 0.05,
        "twinkle": 2,
        "type": "particles",
        "x": 0,
        "y": -0.3
      }
    ],
    "loop": true,
    "name": "Eldritch Magic Circle",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.6
    ],
    "ramps": {
      "alt": "plasma",
      "main": "arcane",
      "smoke": "dark",
      "spark": "arcane"
    },
    "seed": 3,
    "w": 96
  }
});
