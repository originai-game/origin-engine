import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 172,
    "innerLine": 0.75,
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
    "line": 1.25,
    "loop": false,
    "name": "Zapped",
    "outline": "outside",
    "ramps": {
      "main": [
        "#2f5ed8",
        "#62bcff",
        "#dff7ff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.8,
    "seed": 78,
    "supersample": 4,
    "w": 140
  }
});
