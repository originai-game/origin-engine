import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 112,
    "layers": [
      {
        "flick": 0.3,
        "freq": 2,
        "hgt": 1.5,
        "ramp": "$main",
        "rough": 0.3,
        "speed": 1,
        "taper": 0.5,
        "type": "flame",
        "w": 0.6,
        "y": 0.75
      }
    ],
    "line": 1.5,
    "loop": true,
    "name": "Torch Flame",
    "outline": "inside",
    "pivot": [
      0.5,
      0.7286
    ],
    "ramps": {
      "main": [
        "#d1604d",
        "#e39366",
        "#efc172",
        "#f8ebbf"
      ]
    },
    "seed": 92,
    "supersample": 4,
    "w": 64
  }
});
