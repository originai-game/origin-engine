import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.0.3 "下半分glow"
export default file.particles({
  "maxParticles": 6,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 0.1167
    }
  ],
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    22.9412,
    30.0
  ],
  "speedOverLife": [
    [
      0.0,
      1.0207
    ],
    [
      0.1719,
      0.8629
    ],
    [
      0.3438,
      1.172
    ],
    [
      0.5,
      1.8663
    ],
    [
      0.6719,
      3.0845
    ],
    [
      0.6875,
      0.0631
    ],
    [
      0.7031,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.75,
    0.75
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      1.0,
      2.0
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
        0.0,
        0.0,
        1.0,
        1.0
      ]
    ],
    [
      0.6,
      [
        0.0,
        0.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/hisigata0"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk01/Light4/n0-5-0-0-3.effect"
  }
});
