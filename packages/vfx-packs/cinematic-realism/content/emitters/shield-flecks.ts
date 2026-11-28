import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
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
        1,
        2,
        3.200000047683716,
        1
      ]
    ],
    [
      1,
      [
        0.10000000149011612,
        0.30000001192092896,
        1,
        0
      ]
    ]
  ],
  "drag": 4,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.15000000596046448,
    0.4000000059604645
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 82,
    "radius": 0.029999999329447746
  },
  "size": [
    0.012000000104308128,
    0.024000000208616257
  ],
  "speed": [
    1.5,
    4
  ]
});
