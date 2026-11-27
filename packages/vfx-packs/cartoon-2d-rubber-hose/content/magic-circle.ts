import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 80,
    "innerLine": 1.5,
    "layers": [
      {
        "r": 0.95,
        "ramp": "$main",
        "rings": [
          1,
          0.82,
          0.55
        ],
        "runes": 0,
        "spin": 1,
        "squash": 0.35,
        "star": 6,
        "step": 2,
        "ticks": 0,
        "type": "glyph"
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Spell Circle",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#725888",
        "#9f7fa9",
        "#ceb1c1",
        "#edd9cb"
      ]
    },
    "seed": 83,
    "supersample": 4,
    "w": 160
  }
});
