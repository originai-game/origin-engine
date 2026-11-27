import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 160,
    "innerLine": 1.5,
    "layers": [
      {
        "arms": 3,
        "dir": 1,
        "r": 0.85,
        "rIn": 0.25,
        "ramp": "$main",
        "spin": 1,
        "squash": 1.3,
        "twist": 1.2,
        "type": "vortex"
      },
      {
        "count": 8,
        "implodeR": 1,
        "life": [
          0.5,
          0.9
        ],
        "mode": "implode",
        "ramp": "$spark",
        "shape": "star",
        "size": 4,
        "type": "particles"
      }
    ],
    "line": 3,
    "loop": true,
    "name": "Portal",
    "outline": "outside",
    "outlineColor": "#141010",
    "ramps": {
      "main": [
        "#725888",
        "#9f7fa9",
        "#ceb1c1",
        "#edd9cb"
      ],
      "spark": [
        "#d0c1a1",
        "#ebdcb9",
        "#f5e9c9"
      ]
    },
    "seed": 84,
    "supersample": 4,
    "w": 128
  }
});
