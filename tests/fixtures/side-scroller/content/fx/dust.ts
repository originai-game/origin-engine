import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "loop": false,
  "recipe": {
    "fps": 15,
    "frames": 8,
    "h": 24,
    "layers": [
      {
        "count": 4,
        "dir": -90,
        "dissolve": 0.6,
        "emit": 0.1,
        "flat": 0.6,
        "gain": 0.45,
        "life": [
          0.5,
          0.9
        ],
        "r0": 0.12,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.3,
        "spread": 0.5,
        "type": "puffs",
        "x": 0,
        "y": 0.7
      }
    ],
    "name": "Step dust",
    "pivot": [
      0.5,
      0.9
    ],
    "ramps": {
      "alt": "dust",
      "main": "dust",
      "smoke": "dust",
      "spark": "white"
    },
    "seed": 2,
    "w": 40
  }
});
