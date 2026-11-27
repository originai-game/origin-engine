import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 12,
    "h": 64,
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
    "line": 1.5,
    "loop": true,
    "name": "Fuse Sparks",
    "outline": "inside",
    "ramps": {
      "main": [
        "#d6785d",
        "#efa86b",
        "#f1d47e",
        "#fbf5da"
      ]
    },
    "seed": 26,
    "supersample": 4,
    "w": 64
  }
});
