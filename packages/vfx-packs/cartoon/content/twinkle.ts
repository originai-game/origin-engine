import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 128,
    "innerLine": 1,
    "layers": [
      {
        "area": 0.6,
        "count": 4,
        "life": [
          0.5,
          0.8
        ],
        "mode": "ambient",
        "ramp": "$main",
        "shape": "star",
        "size": 16,
        "spanX": 0.62,
        "spanY": 0.62,
        "speed": [
          0,
          0.1
        ],
        "twinkle": 2,
        "type": "particles",
        "wiggle": 0.04
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Twinkle",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "scale": 0.875,
    "seed": 96,
    "supersample": 4,
    "w": 128
  }
});
