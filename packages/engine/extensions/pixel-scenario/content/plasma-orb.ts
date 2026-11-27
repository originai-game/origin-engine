import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 12,
    "h": 48,
    "layers": [
      {
        "count": 8,
        "gain": 0.55,
        "r": 0.95,
        "ramp": "$main",
        "twinkle": 1,
        "type": "rays",
        "width": 0.16
      },
      {
        "pulse": 0.1,
        "r": 0.46,
        "ramp": "$main",
        "rim": 0.3,
        "spin": 1,
        "type": "orb"
      },
      {
        "count": 6,
        "heat": [
          1,
          1
        ],
        "life": 1,
        "mode": "orbit",
        "orbitR": 0.78,
        "ramp": "$spark",
        "shape": "star",
        "size": 1,
        "squash": 0.45,
        "turns": 1,
        "type": "particles"
      },
      {
        "count": 5,
        "drag": 2.4,
        "emit": 0,
        "heat": [
          1,
          0.5
        ],
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 2,
        "speed": [
          1.6,
          2.8
        ],
        "t0": 0,
        "type": "particles",
        "x": 0,
        "y": 0
      }
    ],
    "loop": true,
    "name": "Ion Orb",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "arcane",
      "main": "plasma",
      "smoke": "dark",
      "spark": "plasma"
    },
    "seed": 2,
    "w": 48
  }
});
