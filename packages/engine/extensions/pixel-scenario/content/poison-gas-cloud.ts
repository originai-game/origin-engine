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
        "count": 9,
        "decay": 0.35,
        "dither": 0.6,
        "flat": 0.7,
        "gain": 0.62,
        "life": [
          0.7,
          1
        ],
        "r0": 0.18,
        "r1": 0.4,
        "ramp": "$main",
        "rise": 0.25,
        "spread": 0.55,
        "type": "puffs"
      },
      {
        "area": 0.42,
        "count": 8,
        "emit": 0,
        "heat": [
          0.9,
          0.6
        ],
        "life": [
          0.5,
          0.9
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "bubble",
        "size": [
          1,
          2
        ],
        "speed": [
          0.4,
          0.8
        ],
        "t0": 0.1,
        "type": "particles",
        "wiggle": 0.08,
        "x": 0,
        "y": 0.2
      }
    ],
    "loop": true,
    "name": "Blight Cloud",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "smoke",
      "main": "poison",
      "smoke": "poison",
      "spark": "smoke"
    },
    "seed": 2,
    "w": 64
  }
});
