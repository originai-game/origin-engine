import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 15,
    "frames": 16,
    "h": 188,
    "innerLine": 1,
    "layers": [
      {
        "count": 12,
        "gain": 0.7,
        "r": 1,
        "ramp": "$main",
        "spin": 0.2,
        "t1": 0.8,
        "type": "rays",
        "width": 0.18
      },
      {
        "breakup": 0.3,
        "r0": 0.3,
        "r1": 1,
        "ramp": "$main",
        "t1": 0.7,
        "th": 0.14,
        "type": "ring"
      },
      {
        "area": 0.7,
        "count": 10,
        "emit": 0.5,
        "life": [
          0.45,
          0.8
        ],
        "mode": "rise",
        "ramp": "$spark",
        "shape": "star",
        "size": 5,
        "speed": [
          0.8,
          1.5
        ],
        "twinkle": 2,
        "type": "particles"
      }
    ],
    "line": 2,
    "loop": false,
    "name": "Power Up",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#e09a14",
        "#ffd23c",
        "#fff3a8"
      ],
      "spark": [
        "#d8cfc4",
        "#f4efe8",
        "#ffffff"
      ]
    },
    "scale": 0.820513,
    "seed": 64,
    "supersample": 4,
    "w": 156
  }
});
