import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.05000000074505806
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 36,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.20000000298023224
  },
  "color": [
    6,
    3,
    1.100000023841858,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1.2999999523162842,
        1.100000023841858,
        1,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        0.5,
        0.15000000596046448,
        1
      ]
    ],
    [
      1,
      [
        0.699999988079071,
        0.20000000298023224,
        0.05000000074505806,
        0
      ]
    ]
  ],
  "drag": 1.2000000476837158,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.5,
    1.2000000476837158
  ],
  "looping": false,
  "maxParticles": 48,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.20000000298023224,
    "surface": false
  },
  "size": [
    0.019999999552965164,
    0.03500000014901161
  ],
  "speed": [
    6,
    14
  ],
  "spread": 180
});
