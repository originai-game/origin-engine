import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 80,
    "innerLine": 1,
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
    "line": 2,
    "loop": true,
    "name": "Spell Circle",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#5e34c0",
        "#9466ec",
        "#cfb0ff",
        "#f6eeff"
      ]
    },
    "seed": 83,
    "supersample": 4,
    "w": 160
  }
});
