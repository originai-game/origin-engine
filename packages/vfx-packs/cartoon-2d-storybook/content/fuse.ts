import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 64,
    "innerLine": 0.75,
    "layers": [
      {
        "count": 10,
        "dir": -90,
        "drag": 1.2,
        "gravity": 1.2,
        "life": [
          0.3,
          0.6
        ],
        "mode": "spray",
        "ramp": "$main",
        "shape": "star",
        "size": 4,
        "speed": [
          0.8,
          1.6
        ],
        "spread": 160,
        "type": "particles"
      },
      {
        "pulse": 0.3,
        "pulseRate": 3,
        "r": 0.16,
        "ramp": "$main",
        "spin": 1,
        "type": "orb"
      }
    ],
    "line": 1.25,
    "loop": true,
    "name": "Fuse Sparks",
    "outline": "outside",
    "ramps": {
      "main": [
        "#d2461e",
        "#ff8a24",
        "#ffd040",
        "#fff6c8"
      ]
    },
    "seed": 26,
    "supersample": 4,
    "w": 64
  }
});
