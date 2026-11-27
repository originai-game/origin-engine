import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 172,
    "innerLine": 1.5,
    "layers": [
      {
        "branch": 0.25,
        "count": 5,
        "depth": 4,
        "fadeStart": 0.7,
        "flicker": 0.12,
        "hold": 2,
        "jag": 0.4,
        "len": 0.95,
        "mode": "radial",
        "ramp": "$main",
        "thick": 4,
        "type": "bolt"
      },
      {
        "dur": 0.1,
        "r": 0.45,
        "ramp": "$spark",
        "type": "flash"
      }
    ],
    "line": 3,
    "loop": false,
    "name": "Zapped",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#5d7095",
        "#86aeb4",
        "#dae2c8"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "scale": 0.8,
    "seed": 78,
    "supersample": 4,
    "w": 140
  }
});
