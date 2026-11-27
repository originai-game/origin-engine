import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 172,
    "innerLine": 0.75,
    "layers": [
      {
        "flick": 0.3,
        "freq": 2,
        "hgt": 1.6,
        "ramp": "$main",
        "rough": 0.35,
        "speed": 1,
        "taper": 0.5,
        "type": "flame",
        "w": 0.8,
        "y": 0.9
      },
      {
        "area": 0.3,
        "count": 5,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$alt",
        "shape": "circle",
        "size": 3,
        "speed": [
          0.8,
          1.4
        ],
        "type": "particles",
        "wiggle": 0.3,
        "y": 0.2
      }
    ],
    "line": 1.25,
    "loop": true,
    "name": "Campfire",
    "outline": "outside",
    "pivot": [
      0.5,
      0.7651
    ],
    "ramps": {
      "alt": [
        "#d2461e",
        "#ff8a24",
        "#ffd040",
        "#fff6c8"
      ],
      "main": [
        "#b8321c",
        "#ee6a1e",
        "#ffb22e",
        "#ffe9a0"
      ]
    },
    "scale": 0.774194,
    "seed": 91,
    "supersample": 4,
    "w": 124
  }
});
