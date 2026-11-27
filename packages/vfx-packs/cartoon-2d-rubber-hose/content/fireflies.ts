import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 24,
    "h": 128,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Fireflies",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#94953c",
        "#c7c25e",
        "#ede4a6"
      ]
    },
    "seed": 95,
    "supersample": 4,
    "w": 192
  }
});
