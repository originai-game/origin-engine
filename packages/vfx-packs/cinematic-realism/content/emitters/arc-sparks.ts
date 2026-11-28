import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 16,
      "cycles": 1,
      "interval": 0,
      "time": 0
    },
    {
      "count": 6,
      "cycles": 1,
      "interval": 0,
      "time": 0.6000000238418579
    },
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 1.100000023841858
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.3499999940395355,
    "die": false,
    "friction": 0.30000001192092896
  },
  "colorOverLife": [
    [
      0,
      [
        3,
        3,
        3.4000000953674316,
        1
      ]
    ],
    [
      0.15000000596046448,
      [
        2.5999999046325684,
        1.600000023841858,
        0.6000000238418579,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1.2000000476837158,
        0.4000000059604645,
        0.07999999821186066,
        1
      ]
    ],
    [
      1,
      [
        0.30000001192092896,
        0.05999999865889549,
        0.009999999776482582,
        0
      ]
    ]
  ],
  "drag": 0.800000011920929,
  "duration": 2,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.4000000059604645,
    1
  ],
  "maxParticles": 64,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 70,
    "radius": 0.03999999910593033
  },
  "size": [
    0.012000000104308128,
    0.024000000208616257
  ],
  "speed": [
    1.7999999523162842,
    4.5
  ],
  "trail": {
    "head": false,
    "lifetime": 0.07000000029802322,
    "width": 0.44999998807907104
  }
});
