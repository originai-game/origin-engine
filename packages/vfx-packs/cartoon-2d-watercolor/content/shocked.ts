import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 18,
    "frames": 12,
    "h": 172,
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
    "line": 1.5,
    "loop": false,
    "name": "Zapped",
    "outline": "inside",
    "ramps": {
      "main": [
        "#708cd5",
        "#95cbf3",
        "#e9f8fd"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "scale": 0.8,
    "seed": 78,
    "supersample": 4,
    "w": 140
  }
});
