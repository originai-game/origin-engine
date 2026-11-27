import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 160,
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
    "line": 1.5,
    "loop": true,
    "name": "Portal",
    "outline": "inside",
    "ramps": {
      "main": [
        "#896dc9",
        "#b296e7",
        "#dccaf9",
        "#f7f1fd"
      ],
      "spark": [
        "#e0dbd4",
        "#f6f3ef",
        "#f7f7f7"
      ]
    },
    "seed": 84,
    "supersample": 4,
    "w": 128
  }
});
