import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 8,
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
        1.5,
        0.6000000238418579,
        0
      ]
    ],
    [
      0.15000000596046448,
      [
        2,
        1.5,
        0.6000000238418579,
        0.5
      ]
    ],
    [
      1,
      [
        0.6000000238418579,
        0.4000000059604645,
        0.10000000149011612,
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
  "heightOverLife": [
    [
      0,
      2
    ],
    [
      0.30000001192092896,
      16
    ],
    [
      1,
      10
    ]
  ],
  "lifetime": [
    0.6000000238418579,
    1
  ],
  "looping": false,
  "maxParticles": 10,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0,
      0.15000000596046448
    ]
  },
  "size": [
    0.03999999910593033,
    0.07000000029802322
  ],
  "speed": [
    0,
    0
  ]
});
