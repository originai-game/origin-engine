import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 24,
    "h": 64,
    "layers": [
      {
        "count": 10,
        "gain": 0.5,
        "r": 0.9,
        "ramp": "$main",
        "t1": 0.7,
        "type": "rays",
        "width": 0.18,
        "y": 0.9
      },
      {
        "count": 12,
        "dir": -90,
        "drag": 0.3,
        "emit": 0.3,
        "gravity": 6.5,
        "ground": 1.15,
        "heat": [
          1,
          0.9
        ],
        "life": [
          0.6,
          1
        ],
        "mode": "burst",
        "ramp": "$main",
        "shape": "coin",
        "size": 3,
        "speed": [
          2.6,
          3.6
        ],
        "spread": 70,
        "type": "particles",
        "y": 0.9
      },
      {
        "count": 8,
        "drag": 1.6,
        "emit": 0.6,
        "life": [
          0.35,
          0.7
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": [
          2,
          1
        ],
        "speed": [
          0.6,
          1.4
        ],
        "twinkle": 2,
        "type": "particles",
        "y": 0.6
      }
    ],
    "loop": false,
    "name": "Lucky Loot",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.9125000000000001
    ],
    "ramps": {
      "alt": "holy",
      "main": "gold",
      "smoke": "white",
      "spark": "gold"
    },
    "seed": 2,
    "w": 48
  }
});
