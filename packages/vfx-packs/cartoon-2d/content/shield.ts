import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 144,
    "innerLine": 1,
    "layers": [
      {
        "fill": 0,
        "hex": 0,
        "r": 0.85,
        "ramp": "$main",
        "shimmer": 1,
        "th": 0.09,
        "type": "bubble"
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Shield Bubble",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#2856b8",
        "#4a8ee8",
        "#9cd0ff"
      ]
    },
    "scale": 0.888889,
    "seed": 86,
    "supersample": 4,
    "w": 144
  }
});
