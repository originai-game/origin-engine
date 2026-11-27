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
        "arms": 3,
        "core": "dark",
        "r": 0.86,
        "rIn": 0.2,
        "ramp": "$main",
        "spin": 1,
        "twist": 2.4,
        "type": "vortex"
      },
      {
        "count": 14,
        "heat": [
          0.6,
          1.05
        ],
        "implodeR": 1,
        "life": [
          0.5,
          1
        ],
        "mode": "implode",
        "ramp": "$spark",
        "shape": "dot",
        "size": 1,
        "swirl": 1.4,
        "type": "particles"
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
        "spread": 0.42,
        "t0": 0.15,
        "type": "puffs",
        "x": 0,
        "y": 0
      },
      {
        "area": 0.48999999999999994,
        "count": 3,
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
        "y": 0
      }
    ],
    "loop": true,
    "name": "Shadow Vortex",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "void",
      "main": "hue:305",
      "smoke": "shadow",
      "spark": "hue:305"
    },
    "seed": 4,
    "w": 64
  }
});
