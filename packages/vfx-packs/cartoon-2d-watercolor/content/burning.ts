import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 192,
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
    "line": 1.5,
    "loop": true,
    "name": "On Fire",
    "outline": "inside",
    "pivot": [
      0.5,
      0.7875
    ],
    "ramps": {
      "main": [
        "#d1604d",
        "#e39366",
        "#efc172",
        "#f8ebbf"
      ]
    },
    "scale": 0.75,
    "seed": 76,
    "supersample": 4,
    "w": 128
  }
});
