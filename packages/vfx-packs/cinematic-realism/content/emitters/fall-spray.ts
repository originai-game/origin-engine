import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.019999999552965164
  },
  "blend": "alpha",
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
        0.800000011920929
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        1,
        1,
        0.6000000238418579
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
  "duration": 2,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.5,
    1
  ],
  "maxParticles": 160,
  "rate": 120,
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.800000011920929
  },
  "size": [
    0.012000000104308128,
    0.029999999329447746
  ],
  "speed": [
    2,
    4.5
  ]
});
