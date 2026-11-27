import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 80,
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
    "line": 1.5,
    "loop": true,
    "name": "Spell Circle",
    "outline": "inside",
    "ramps": {
      "main": [
        "#896dc9",
        "#b296e7",
        "#dccaf9",
        "#f7f1fd"
      ]
    },
    "seed": 83,
    "supersample": 4,
    "w": 160
  }
});
