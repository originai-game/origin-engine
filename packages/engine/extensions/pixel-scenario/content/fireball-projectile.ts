import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 8,
    "h": 32,
    "layers": [
      {
        "area": 0.15,
        "count": 10,
        "dir": 180,
        "drag": 1,
        "heat": [
          1,
          0.4
        ],
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "dot",
        "size": [
          2,
          1
        ],
        "speed": [
          1.6,
          3
        ],
        "spread": 50,
        "type": "particles",
        "x": 1,
        "y": 0
      },
      {
        "len": 2.6,
        "r": 0.5,
        "ramp": "$main",
        "rough": 0.6,
        "speed": 2,
        "type": "comet",
        "x": 1
      },
      {
        "area": 0.4,
        "count": 6,
        "emit": 0,
        "heat": [
          1,
          0.35
        ],
        "life": [
          0.4,
          0.9
        ],
        "mode": "rise",
        "ramp": "$spark",
        "speed": [
          0.6,
          1.3
        ],
        "t0": 0,
        "type": "particles",
        "wiggle": 0.07,
        "x": 0.2,
        "y": 0
      }
    ],
    "loop": true,
    "name": "Ember Projectile",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.78,
      0.5
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 1,
    "w": 64
  }
});
