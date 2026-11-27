import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
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
    0.36000001430511475,
    0.019999999552965164,
    0.019999999552965164,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.8500000238418579,
      [
        1,
        1,
        1,
        1
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
  "drag": 1.2000000476837158,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.4000000059604645,
    0.800000011920929
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 40,
    "radius": 0.019999999552965164
  },
  "size": [
    0.00800000037997961,
    0.01600000075995922
  ],
  "speed": [
    1.5,
    3.799999952316284
  ]
});
