import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.019999999552965164
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 12,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 0
  },
  "color": [
    0.8799999952316284,
    0.9399999976158142,
    1,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        1,
        1,
        0.6499999761581421
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 0.30000001192092896,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.3499999940395355,
    0.699999988079071
  ],
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 12,
    "radius": 0.019999999552965164
  },
  "size": [
    0.009999999776482582,
    0.02199999988079071
  ],
  "speed": [
    1.5,
    3
  ]
});
