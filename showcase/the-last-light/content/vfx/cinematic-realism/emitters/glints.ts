import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 12,
      "cycles": 1,
      "interval": 0,
      "time": 0.05000000074505806
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        3,
        2,
        0.699999988079071,
        1
      ]
    ],
    [
      1,
      [
        0.800000011920929,
        0.3499999940395355,
        0.07999999821186066,
        0
      ]
    ]
  ],
  "drag": 1.5,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    1,
    0
  ],
  "lifetime": [
    0.4000000059604645,
    0.800000011920929
  ],
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.20000000298023224,
    "surface": false
  },
  "size": [
    0.014999999664723873,
    0.029999999329447746
  ],
  "speed": [
    1,
    2
  ],
  "spread": 20,
  "trail": {
    "head": true,
    "lifetime": 0.15000000596046448,
    "width": 0.6000000238418579
  }
});
