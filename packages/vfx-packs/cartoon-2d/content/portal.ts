import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 160,
    "innerLine": 1,
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
    "line": 2,
    "loop": true,
    "name": "Portal",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#5e34c0",
        "#9466ec",
        "#cfb0ff",
        "#f6eeff"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "seed": 84,
    "supersample": 4,
    "w": 128
  }
});
