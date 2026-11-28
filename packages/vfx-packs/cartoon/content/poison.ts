import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 168,
    "innerLine": 1,
    "layers": [
      {
        "area": 0.6,
        "count": 8,
        "life": [
          0.6,
          0.95
        ],
        "mode": "rise",
        "ramp": "$main",
        "shape": "bubble",
        "size": [
          3,
          6
        ],
        "speed": [
          0.6,
          1.2
        ],
        "type": "particles",
        "wiggle": 0.3,
        "y": 0.6
      },
      {
        "count": 3,
        "decay": 0.6,
        "gain": 0.6,
        "life": [
          0.6,
          0.95
        ],
        "r0": 0.1,
        "r1": 0.2,
        "ramp": "$main",
        "rise": 0.4,
        "spread": 0.4,
        "type": "puffs",
        "y": 0.4
      }
    ],
    "line": 2,
    "loop": true,
    "name": "Poisoned",
    "outline": "outside",
    "outlineColor": "#3a2416",
    "ramps": {
      "main": [
        "#4e7e1a",
        "#8cc236",
        "#d0ee76"
      ]
    },
    "scale": 0.8,
    "seed": 75,
    "supersample": 4,
    "w": 120
  }
});
