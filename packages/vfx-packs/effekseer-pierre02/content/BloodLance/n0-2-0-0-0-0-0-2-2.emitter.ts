import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.0.0.2.2 "EmberVariation"
export default file.particles({
  "maxParticles": 1500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0717,
      1.6125,
      0.0717
    ]
  },
  "spread": 0.0,
  "rate": 1800.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3,
    0.9
  ],
  "speed": [
    0.215,
    0.645
  ],
  "size": [
    2.8667,
    2.8667
  ],
  "sizeOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.0781,
      0.2197
    ],
    [
      0.3125,
      0.4714
    ],
    [
      0.5156,
      0.6553
    ],
    [
      0.75,
      0.8357
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.0781,
      0.0032
    ],
    [
      0.3125,
      0.3204
    ],
    [
      0.5156,
      0.5568
    ],
    [
      0.75,
      0.7888
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        0.0055,
        0.0104,
        0.502
      ]
    ],
    [
      0.4167,
      [
        0.2195,
        0.0055,
        0.0104,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-2-2.effect"
  }
});
