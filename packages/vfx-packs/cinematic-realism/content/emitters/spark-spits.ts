import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.02500000037252903
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0.30000001192092896
    },
    {
      "count": 7,
      "cycles": 1,
      "interval": 0,
      "time": 1.399999976158142
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
  "drag": 0.800000011920929,
  "duration": 2,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.4000000059604645,
    0.8999999761581421
  ],
  "maxParticles": 32,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.05000000074505806
  },
  "size": [
    0.009999999776482582,
    0.019999999552965164
  ],
  "speed": [
    1.5,
    4
  ],
  "trail": {
    "head": false,
    "lifetime": 0.05999999865889549,
    "width": 0.3499999940395355
  }
});
