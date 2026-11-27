import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 20,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        2,
        2.0999999046325684,
        2.200000047683716,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1.0499999523162842,
        1.100000023841858,
        0.800000011920929
      ]
    ],
    [
      1,
      [
        0.20000000298023224,
        0.20000000298023224,
        0.20000000298023224,
        0
      ]
    ]
  ],
  "drag": 2,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -6,
    0
  ],
  "lifetime": [
    0.15000000596046448,
    0.5
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.05000000074505806
  },
  "size": [
    0.006000000052154064,
    0.012000000104308128
  ],
  "speed": [
    1,
    4
  ]
});
