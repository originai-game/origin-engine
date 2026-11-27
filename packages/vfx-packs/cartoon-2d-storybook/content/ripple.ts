import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": false,
    "fps": 12,
    "frames": 16,
    "h": 48,
    "innerLine": 0.75,
    "layers": [
      {
        "breakup": 0.15,
        "fade": 1,
        "r0": 0.2,
        "r1": 1.6,
        "ramp": "$main",
        "squash": 0.22,
        "th": 0.12,
        "type": "ring"
      },
      {
        "breakup": 0.15,
        "fade": 1,
        "r0": 0.2,
        "r1": 1.6,
        "ramp": "$main",
        "squash": 0.22,
        "t0": 0.5,
        "t1": 1.5,
        "th": 0.12,
        "type": "ring"
      }
    ],
    "line": 1.25,
    "loop": true,
    "name": "Ripples",
    "outline": "outside",
    "ramps": {
      "main": [
        "#7cc4ee",
        "#c6ecff",
        "#ffffff"
      ]
    },
    "seed": 53,
    "supersample": 4,
    "w": 128
  }
});
