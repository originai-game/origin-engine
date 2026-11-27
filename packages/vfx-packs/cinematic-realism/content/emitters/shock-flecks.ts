import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    },
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0.11999999731779099
    },
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0.30000001192092896
    },
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0.4399999976158142
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1.399999976158142,
        2.4000000953674316,
        4,
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
  "duration": 0.6000000238418579,
  "lifetime": [
    0.11999999731779099,
    0.30000001192092896
  ],
  "maxParticles": 48,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.25,
    "surface": false
  },
  "size": [
    0.012000000104308128,
    0.024000000208616257
  ],
  "speed": [
    1.2000000476837158,
    3
  ]
});
