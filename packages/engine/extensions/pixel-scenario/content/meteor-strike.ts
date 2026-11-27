import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 26,
    "h": 96,
    "impact": 0.36,
    "layers": [
      {
        "from": [
          -0.85,
          -0.95
        ],
        "len": 0.95,
        "r": 0.17,
        "ramp": "$main",
        "speed": 3,
        "type": "comet",
        "until": 0.36,
        "x": 0.2,
        "y": 0.55
      },
      {
        "count": 8,
        "emit": 0.15,
        "flat": 0.3,
        "gain": 0.6,
        "life": [
          0.5,
          0.8
        ],
        "r0": 0.12,
        "r1": 0.3,
        "ramp": "$smoke",
        "rise": 0.35,
        "spread": 0.5,
        "t0": 0.42,
        "type": "puffs",
        "x": 0.2,
        "y": 0.5
      },
      {
        "count": 7,
        "grow": 0.15,
        "len": 0.8,
        "ramp": "$alt",
        "squash": 0.3,
        "t0": 0.36,
        "type": "cracks",
        "x": 0.2,
        "y": 0.62
      },
      {
        "cool": 0.6,
        "erode": 1,
        "grow": 0.35,
        "r0": 0.1,
        "r1": 0.55,
        "ramp": "$main",
        "squash": 0.8,
        "t0": 0.36,
        "type": "burst",
        "x": 0.2,
        "y": 0.5
      },
      {
        "breakup": 0.7,
        "r0": 0.2,
        "r1": 1,
        "ramp": "$main",
        "squash": 0.32,
        "t0": 0.36,
        "t1": 0.8,
        "th": 0.16,
        "type": "ring",
        "x": 0.2,
        "y": 0.62
      },
      {
        "count": 10,
        "dir": -90,
        "drag": 0.6,
        "gravity": 4,
        "heat": [
          0.9,
          0.6
        ],
        "life": [
          0.35,
          0.6
        ],
        "mode": "burst",
        "ramp": "$alt",
        "shape": "chunk",
        "size": [
          2,
          1
        ],
        "speed": [
          1.2,
          2.2
        ],
        "spread": 150,
        "t0": 0.36,
        "type": "particles",
        "x": 0.2,
        "y": 0.55
      },
      {
        "dur": 0.1,
        "r": 0.45,
        "ramp": "$main",
        "t0": 0.36,
        "type": "flash",
        "x": 0.2,
        "y": 0.5
      }
    ],
    "loop": false,
    "name": "Ember Meteor",
    "outline": "none",
    "palette": null,
    "pivot": [
      0.6,
      0.8099999999999999
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 9,
    "w": 96
  }
});
