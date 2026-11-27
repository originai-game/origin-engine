import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.019999999552965164
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 90,
      "cycles": 1,
      "interval": 0,
      "time": 0.019999999552965164
    },
    {
      "count": 50,
      "cycles": 1,
      "interval": 0,
      "time": 0.10000000149011612
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
  "duration": 0.15000000596046448,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.5,
    1.100000023841858
  ],
  "looping": false,
  "maxParticles": 160,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 38,
    "radius": 0.20000000298023224
  },
  "size": [
    0.012000000104308128,
    0.029999999329447746
  ],
  "speed": [
    1.7999999523162842,
    3.5999999046325684
  ]
});
