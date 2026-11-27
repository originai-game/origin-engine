import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "loop": false,
  "recipe": {
    "fps": 18,
    "frames": 12,
    "h": 96,
    "impact": 0.08,
    "layers": [
      {
        "erode": 0.7,
        "r": 0.8,
        "ramp": "$main",
        "sweep": 0.2,
        "t1": 0.7,
        "th": 0.22,
        "trail": 0.5,
        "type": "slash"
      },
      {
        "hollow": 0.9,
        "inner": 0.2,
        "points": 4,
        "pop": 0.2,
        "r": 0.3,
        "ramp": "$alt",
        "sharp": 0.9,
        "t1": 0.4,
        "type": "star",
        "x": 0.55,
        "y": 0.2
      },
      {
        "count": 10,
        "dir": -40,
        "gravity": 1.4,
        "heat": 0.8,
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$alt",
        "shape": "shard",
        "size": 2,
        "speed": 1.1,
        "spread": 160,
        "type": "particles",
        "x": 0.55,
        "y": 0.2
      }
    ],
    "name": "Blade hit",
    "outline": "outside",
    "outlineColor": "#0b0c14",
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "white",
      "main": "steel",
      "smoke": "dust",
      "spark": "white"
    },
    "seed": 4,
    "w": 112
  }
});
