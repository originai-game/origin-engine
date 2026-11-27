import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 16,
    "h": 48,
    "layers": [
      {
        "cap": 0.5,
        "gain": 1.05,
        "len": 6.6,
        "ramp": "$main",
        "speed": 3,
        "th": 0.36,
        "type": "beam",
        "wob": 0.55,
        "x": -3.1166666666666663
      },
      {
        "area": 0.15,
        "count": 18,
        "dir": 0,
        "drag": 0,
        "emit": 0,
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
        "shape": "streak",
        "size": 2,
        "speed": [
          5,
          8
        ],
        "spread": 16,
        "type": "particles",
        "x": -2.8666666666666663,
        "y": 0
      },
      {
        "pulse": 0.12,
        "r": 0.42,
        "ramp": "$main",
        "spin": 1,
        "type": "orb",
        "x": -3.1166666666666663
      },
      {
        "count": 7,
        "drag": 2.4,
        "emit": 0.3,
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
        "x": 0.4,
        "y": 0
      }
    ],
    "loop": true,
    "name": "Aether Beam",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.0750000000000001,
      0.5
    ],
    "ramps": {
      "alt": "lightning",
      "main": "hue:255",
      "smoke": "smoke",
      "spark": "hue:255"
    },
    "seed": 2,
    "w": 176
  }
});
