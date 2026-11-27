import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 12,
    "frames": 16,
    "h": 64,
    "layers": [
      {
        "dither": 0.9,
        "fill": 0.15,
        "hex": 0.42,
        "r": 0.86,
        "ramp": "$main",
        "shimmer": 1,
        "th": 0.06,
        "type": "bubble"
      },
      {
        "both": true,
        "count": 5,
        "heat": [
          1,
          1
        ],
        "life": 1,
        "mode": "orbit",
        "orbitR": 0.86,
        "ramp": "$spark",
        "shape": "star",
        "size": 1,
        "turns": 1,
        "type": "particles"
      }
    ],
    "loop": true,
    "name": "Flux Shield",
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
    "seed": 1,
    "w": 64
  }
});
