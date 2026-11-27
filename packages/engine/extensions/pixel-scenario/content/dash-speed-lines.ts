import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 18,
    "frames": 10,
    "h": 48,
    "layers": [
      {
        "count": 4,
        "dir": 180,
        "emit": 0.25,
        "flat": 0.2,
        "gain": 0.6,
        "life": [
          0.5,
          0.8
        ],
        "r0": 0.08,
        "r1": 0.2,
        "ramp": "$smoke",
        "rise": 0.15,
        "speed": [
          0.3,
          0.6
        ],
        "spread": 0.3,
        "type": "puffs",
        "x": -1,
        "y": 0.75
      },
      {
        "angle": 180,
        "count": 10,
        "len": 1.2,
        "mode": "speed",
        "ramp": "$main",
        "spanY": 1.3,
        "speed": 2,
        "type": "streaks"
      }
    ],
    "loop": false,
    "name": "Cyclone Dash",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "white",
      "main": "wind",
      "smoke": "white",
      "spark": "wind"
    },
    "seed": 2,
    "w": 64
  }
});
