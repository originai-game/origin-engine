import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 14,
    "h": 48,
    "layers": [
      {
        "count": 8,
        "decay": 0.45,
        "emit": 0.1,
        "flat": 0.8,
        "gain": 0.8,
        "life": [
          0.6,
          1
        ],
        "r0": 0.2,
        "r1": 0.44,
        "ramp": "$smoke",
        "rise": 0.22,
        "spread": 0.3,
        "type": "puffs"
      },
      {
        "gain": 0.7,
        "r0": 0.2,
        "r1": 0.9,
        "ramp": "$smoke",
        "t1": 0.5,
        "th": 0.1,
        "type": "ring"
      },
      {
        "count": 4,
        "drag": 2,
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 2,
        "speed": [
          0.8,
          1.4
        ],
        "t0": 0.1,
        "type": "particles"
      }
    ],
    "loop": false,
    "name": "Ashen Poof",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "dust",
      "main": "smoke",
      "smoke": "smoke",
      "spark": "white"
    },
    "seed": 3,
    "w": 48
  }
});
