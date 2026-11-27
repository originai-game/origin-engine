import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 128,
    "layers": [
      {
        "area": 0.3,
        "count": 8,
        "life": [
          0.6,
          1
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "bubble",
        "size": [
          3,
          6
        ],
        "speed": [
          0.8,
          1.4
        ],
        "type": "particles",
        "wiggle": 0.3,
        "y": 0.8
      }
    ],
    "line": 1.5,
    "loop": true,
    "name": "Bubbles",
    "outline": "inside",
    "ramps": {
      "main": [
        "#a5d0ea",
        "#d9effb",
        "#f7f7f7"
      ]
    },
    "seed": 54,
    "supersample": 4,
    "w": 64
  }
});
