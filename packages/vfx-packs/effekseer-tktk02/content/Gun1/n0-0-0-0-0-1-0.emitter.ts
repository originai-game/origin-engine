import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.1.0 "LineColor"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 3000.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    4.125,
    18.975
  ],
  "speedOverLife": [
    [
      0.0,
      2.9785
    ],
    [
      0.1562,
      2.1316
    ],
    [
      0.2969,
      1.4897
    ],
    [
      0.5,
      0.7996
    ],
    [
      0.7031,
      0.3896
    ],
    [
      0.7188,
      0.0808
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    3.0938,
    3.7812
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      0.5
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.1564,
        0.034,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.4298,
        0.1564,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
