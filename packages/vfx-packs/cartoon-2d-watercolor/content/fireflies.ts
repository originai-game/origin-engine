import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 128,
    "layers": [
      {
        "count": 10,
        "glow": 1,
        "life": [
          0.8,
          1
        ],
        "mode": "ambient",
        "ramp": "$main",
        "shape": "circle",
        "size": 9,
        "spanX": 0.85,
        "spanY": 0.6,
        "speed": [
          0,
          0.15
        ],
        "twinkle": 1,
        "type": "particles",
        "wiggle": 0.2
      }
    ],
    "line": 1.5,
    "loop": true,
    "name": "Fireflies",
    "outline": "inside",
    "ramps": {
      "main": [
        "#c4d14d",
        "#e1e784",
        "#f8fbda"
      ]
    },
    "seed": 95,
    "supersample": 4,
    "w": 192
  }
});
