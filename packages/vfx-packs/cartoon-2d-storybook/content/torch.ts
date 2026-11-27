import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 112,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": true,
    "name": "Torch Flame",
    "outline": "outside",
    "pivot": [
      0.5,
      0.7286
    ],
    "ramps": {
      "main": [
        "#b8321c",
        "#ee6a1e",
        "#ffb22e",
        "#ffe9a0"
      ]
    },
    "seed": 92,
    "supersample": 4,
    "w": 64
  }
});
