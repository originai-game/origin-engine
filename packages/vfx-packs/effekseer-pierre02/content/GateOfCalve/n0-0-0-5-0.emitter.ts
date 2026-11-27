import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.5.0 "Calve"
export default file.particles({
  "maxParticles": 113,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      0.001,
      12.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 90,
      "interval": 0.0167
    }
  ],
  "duration": 1.5,
  "looping": false,
  "lifetime": [
    10.0,
    10.0
  ],
  "speed": [
    1.41,
    1.41
  ],
  "speedOverLife": [
    [
      0.0,
      16.1361
    ],
    [
      0.0156,
      18.1869
    ],
    [
      0.0312,
      17.9053
    ],
    [
      0.0469,
      14.0048
    ],
    [
      0.0625,
      5.8349
    ],
    [
      0.0781,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.6,
    1.6
  ],
  "heightOverLife": [
    [
      0.0,
      0.4
    ],
    [
      1.0,
      0.4
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      1.0,
      0.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.975,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Yakiniku"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/GateOfCalve/n0-0-0-5-0.effect"
  }
});
