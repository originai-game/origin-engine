import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 16,
    "h": 64,
    "layers": [
      {
        "arms": 2,
        "core": "dark",
        "r": 0.6,
        "rIn": 0.28,
        "ramp": "$main",
        "spin": 1,
        "squash": 1.55,
        "twist": 1.8,
        "type": "vortex",
        "y": 0
      },
      {
        "count": 12,
        "heat": [
          0.6,
          1.05
        ],
        "implodeR": 0.85,
        "life": [
          0.5,
          1
        ],
        "mode": "implode",
        "ramp": "$spark",
        "shape": "dot",
        "squash": 1.5,
        "swirl": 1.2,
        "type": "particles",
        "y": 0
      },
      {
        "area": 0.6,
        "areaY": 0.2,
        "count": 6,
        "heat": [
          1,
          0.5
        ],
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          0.5,
          1
        ],
        "type": "particles",
        "y": 0.6
      }
    ],
    "loop": true,
    "name": "Astral Portal",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.75
    ],
    "ramps": {
      "alt": "plasma",
      "main": "arcane",
      "smoke": "dark",
      "spark": "arcane"
    },
    "seed": 2,
    "w": 64
  }
});
