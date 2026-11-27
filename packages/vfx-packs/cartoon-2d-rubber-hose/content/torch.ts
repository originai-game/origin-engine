import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 112,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Torch Flame",
    "outline": "outside",
    "outlineColor": "#141010",
    "pivot": [
      0.5,
      0.7286
    ],
    "ramps": {
      "main": [
        "#9d523c",
        "#c27746",
        "#d2a051",
        "#e8d090"
      ]
    },
    "seed": 92,
    "supersample": 4,
    "w": 64
  }
});
