import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 144,
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
    "line": 1.5,
    "loop": true,
    "name": "Shield Bubble",
    "outline": "inside",
    "ramps": {
      "main": [
        "#5d80cb",
        "#83ace2",
        "#bcdbf8"
      ]
    },
    "scale": 0.888889,
    "seed": 86,
    "supersample": 4,
    "w": 144
  }
});
