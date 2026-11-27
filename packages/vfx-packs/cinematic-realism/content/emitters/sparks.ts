import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.02500000037252903
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 22,
      "cycles": 1,
      "interval": 0,
      "time": 0
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
        2,
        1.2000000476837158,
        0.44999998807907104,
        1
      ]
    ],
    [
      0.5,
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
  "drag": 1.2999999523162842,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.18000000715255737,
    0.6499999761581421
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 65,
    "radius": 0.02500000037252903
  },
  "size": [
    0.012000000104308128,
    0.024000000208616257
  ],
  "speed": [
    1.5,
    4
  ],
  "trail": {
    "head": false,
    "lifetime": 0.07000000029802322,
    "width": 0.3499999940395355
  }
});
