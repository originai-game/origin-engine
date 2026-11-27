import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 18,
    "h": 160,
    "innerLine": 1,
    "layers": [
      {
        "count": 9,
        "gain": 0.55,
        "jitter": 0.2,
        "r": 1.1,
        "ramp": "$smoke",
        "t1": 0.85,
        "type": "rays",
        "width": 0.2,
        "y": 0.2
      },
      {
        "area": 0.5,
        "count": 8,
        "emit": 0.6,
        "life": [
          0.4,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          0.6,
          1.2
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 0.3
      },
      {
        "count": 8,
        "dir": -90,
        "drag": 0.3,
        "emit": 0.2,
        "gravity": 4.5,
        "ground": 0.55,
        "life": [
          0.6,
          0.95
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "coin",
        "size": 8,
        "speed": [
          2,
          2.8
        ],
        "spread": 70,
        "type": "particles",
        "y": 0.4
      }
    ],
    "line": 2,
    "loop": false,
    "name": "Treasure",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "pivot": [
      0.5,
      0.74
    ],
    "ramps": {
      "main": [
        "#b8700f",
        "#ecaa1e",
        "#ffdc58",
        "#fff6c8"
      ],
      "smoke": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ],
      "spark": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ]
    },
    "seed": 66,
    "supersample": 4,
    "w": 128
  }
});
