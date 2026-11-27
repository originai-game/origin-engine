import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0.0.0.1 "ReimsColor"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "sphere",
    "radius": 5.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 7
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.6667,
    1.6667
  ],
  "heightOverLife": [
    [
      0.0,
      0.55
    ],
    [
      1.0,
      0.55
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
        0.2195,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      0.8,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.5356,
        0.5356,
        1.0,
        1.0
      ]
    ],
    [
      0.8,
      [
        0.5356,
        0.5356,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-1.effect"
  }
});
