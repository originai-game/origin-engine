import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 192,
    "innerLine": 0.75,
    "layers": [
      {
        "flick": 0.3,
        "freq": 2,
        "hgt": 2,
        "ramp": "$main",
        "rough": 0.35,
        "speed": 1,
        "taper": 0.4,
        "type": "flame",
        "w": 0.75,
        "y": 1.1
      },
      {
        "area": 0.5,
        "count": 5,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "circle",
        "size": 3,
        "speed": [
          0.8,
          1.4
        ],
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 1.25,
    "loop": true,
    "name": "On Fire",
    "outline": "outside",
    "pivot": [
      0.5,
      0.7875
    ],
    "ramps": {
      "main": [
        "#b8321c",
        "#ee6a1e",
        "#ffb22e",
        "#ffe9a0"
      ]
    },
    "scale": 0.75,
    "seed": 76,
    "supersample": 4,
    "w": 128
  }
});
