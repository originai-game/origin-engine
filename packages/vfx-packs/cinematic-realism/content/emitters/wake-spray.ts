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
    0.4000000059604645,
    0.800000011920929
  ],
  "maxParticles": 96,
  "rate": 80,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.4000000059604645
  },
  "size": [
    0.01600000075995922,
    0.035999998450279236
  ],
  "speed": [
    1.5,
    3.4000000953674316
  ]
});
