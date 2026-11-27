import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 16,
    "h": 112,
    "layers": [
      {
        "flick": 0.4,
        "freq": 2.8,
        "gain": 1.15,
        "hgt": 2.05,
        "hollow": 0.8,
        "ramp": "$main",
        "speed": 2,
        "taper": 0.4,
        "type": "flame",
        "w": 0.62,
        "y": 1
      },
      {
        "area": 0.55,
        "areaY": 0.25,
        "count": 14,
        "heat": [
          1.05,
          0.4
        ],
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "size": [
          2,
          1
        ],
        "speed": [
          1,
          1.9
        ],
        "type": "particles",
        "wiggle": 0.05,
        "x": 0,
        "y": 0.9
      },
      {
        "blend": "under",
        "count": 5,
        "decay": 0.4,
        "emit": 0.5,
        "gain": 0.75,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.06,
        "r1": 0.16,
        "ramp": "$smoke",
        "rise": 0.5,
        "spread": 0.36,
        "t0": 0.15,
        "type": "puffs",
        "x": 0,
        "y": 0.3
      },
      {
        "area": 0.42,
        "count": 4,
        "emit": 0,
        "heat": [
          0.95,
          0.4
        ],
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$main",
        "speed": [
          0.5,
          1
        ],
        "t0": 0.1,
        "type": "particles",
        "wiggle": 0.05,
        "x": 0,
        "y": 0.3
      }
    ],
    "loop": true,
    "name": "Night Aura",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.9285714285714286
    ],
    "ramps": {
      "alt": "void",
      "main": "dark",
      "smoke": "shadow",
      "spark": "dark"
    },
    "seed": 3,
    "w": 96
  }
});
