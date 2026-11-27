import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 18,
    "frames": 12,
    "h": 64,
    "layers": [
      {
        "a0": -125,
        "a1": 50,
        "angle": -12,
        "gain": 0.8,
        "r": 0.56,
        "ramp": "$main",
        "seed": 3,
        "squash": 0.85,
        "sweep": 0.32,
        "t0": 0.03,
        "t1": 0.6,
        "th": 0.1,
        "trail": 0.8,
        "type": "slash"
      },
      {
        "a0": -150,
        "a1": 50,
        "angle": -12,
        "r": 0.76,
        "ramp": "$main",
        "squash": 0.85,
        "sweep": 0.3,
        "t0": 0,
        "t1": 0.7,
        "th": 0.24,
        "trail": 0.95,
        "type": "slash"
      },
      {
        "count": 5,
        "dir": 140,
        "drag": 3,
        "heat": [
          1,
          0.5
        ],
        "life": [
          0.3,
          0.55
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "streak",
        "size": 2,
        "speed": [
          1.2,
          2.2
        ],
        "spread": 70,
        "t0": 0.12,
        "t1": 0.75,
        "type": "particles",
        "x": 0.44995132678057753,
        "y": 0.4596266658713868
      },
      {
        "count": 6,
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
        "t0": 0.15,
        "type": "particles",
        "x": 0,
        "y": 0
      }
    ],
    "loop": false,
    "name": "Razor Slash",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "white",
      "main": "steel",
      "smoke": "dust",
      "spark": "white"
    },
    "seed": 2,
    "w": 64
  }
});
