import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.07999999821186066
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 40,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1.399999976158142,
        2,
        3.200000047683716,
        1
      ]
    ],
    [
      1,
      [
        0.20000000298023224,
        0.4000000059604645,
        1.2000000476837158,
        0
      ]
    ]
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.15000000596046448,
    0.30000001192092896
  ],
  "looping": false,
  "maxParticles": 48,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 3,
    "radius": 0.6000000238418579
  },
  "size": [
    0.019999999552965164,
    0.03999999910593033
  ],
  "space": "local",
  "speed": [
    30,
    60
  ]
});
