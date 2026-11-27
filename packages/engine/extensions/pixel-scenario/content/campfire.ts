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
        "count": 3,
        "decay": 0.5,
        "gain": 0.45,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.08,
        "r1": 0.22,
        "ramp": "$smoke",
        "rise": 0.8,
        "spread": 0.2,
        "type": "puffs",
        "x": 0,
        "y": -0.6
      },
      {
        "flick": 0.4,
        "hgt": 2.45,
        "ramp": "$main",
        "rough": 0.8,
        "speed": 2,
        "type": "flame",
        "w": 0.78,
        "y": 1.3
      },
      {
        "area": 0.4,
        "count": 6,
        "heat": [
          1,
          0.35
        ],
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "speed": [
          0.8,
          1.5
        ],
        "type": "particles",
        "wiggle": 0.08,
        "y": 0.2
      }
    ],
    "loop": true,
    "name": "Ember Flame",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.9333333333333332
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 1,
    "w": 32
  }
});
