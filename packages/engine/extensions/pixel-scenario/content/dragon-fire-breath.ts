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
        "cone": 20,
        "count": 5,
        "dir": 0,
        "gain": 0.45,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.08,
        "r1": 0.25,
        "ramp": "$smoke",
        "rise": 0.4,
        "speed": [
          1.4,
          2
        ],
        "spread": 0.1,
        "type": "puffs",
        "x": -0.4,
        "y": -0.1
      },
      {
        "cone": 12,
        "count": 36,
        "decay": 0.45,
        "dir": 0,
        "dissolve": 0.5,
        "flat": 0.5,
        "freq": 2.6,
        "gain": 1.08,
        "life": [
          0.5,
          0.62
        ],
        "r0": 0.1,
        "r1": 0.44,
        "ramp": "$main",
        "rise": 0.05,
        "speed": [
          2.5,
          2.75
        ],
        "spread": 0.02,
        "type": "puffs",
        "x": -1.6,
        "y": 0
      },
      {
        "count": 10,
        "dir": 0,
        "drag": 0.5,
        "life": [
          0.3,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "dot",
        "size": [
          2,
          1
        ],
        "speed": [
          2.5,
          4
        ],
        "spread": 40,
        "type": "particles",
        "x": -1.55
      }
    ],
    "loop": true,
    "name": "Cinder Breath",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.1,
      0.5
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 3,
    "w": 96
  }
});
