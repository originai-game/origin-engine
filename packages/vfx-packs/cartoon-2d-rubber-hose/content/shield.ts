import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 144,
    "innerLine": 1.5,
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
    "line": 3,
    "loop": true,
    "name": "Shield Bubble",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#526682",
        "#728fa3",
        "#adc1be"
      ]
    },
    "scale": 0.888889,
    "seed": 86,
    "supersample": 4,
    "w": 144
  }
});
