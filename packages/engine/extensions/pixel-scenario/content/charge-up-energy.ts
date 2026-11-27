import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 16,
    "h": 48,
    "layers": [
      {
        "breakup": 0,
        "ease": "inQuad",
        "fade": 0,
        "gain": 0.7,
        "r0": 0.95,
        "r1": 0.35,
        "ramp": "$main",
        "th": 0.06,
        "thEnd": 0.1,
        "type": "ring"
      },
      {
        "pulse": 0.15,
        "pulseRate": 2,
        "r": 0.3,
        "ramp": "$main",
        "spin": 1,
        "type": "orb"
      },
      {
        "count": 18,
        "heat": [
          0.6,
          1.05
        ],
        "implodeR": 1,
        "life": [
          0.5,
          0.9
        ],
        "mode": "implode",
        "ramp": "$spark",
        "shape": "streak",
        "size": 1,
        "swirl": 0.8,
        "type": "particles"
      }
    ],
    "loop": true,
    "name": "Pulse Charge",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "lightning",
      "main": "energy",
      "smoke": "smoke",
      "spark": "energy"
    },
    "seed": 8,
    "w": 48
  }
});
