import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 128,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Bubbles",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#95b4ae",
        "#c9d7c4",
        "#f5e9c9"
      ]
    },
    "seed": 54,
    "supersample": 4,
    "w": 64
  }
});
