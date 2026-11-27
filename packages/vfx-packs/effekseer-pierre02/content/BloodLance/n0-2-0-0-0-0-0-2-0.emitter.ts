import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.0.0.2.0 "EmberVariation"
export default file.particles({
  "maxParticles": 1725,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0717,
      1.6125,
      0.0717
    ]
  },
  "spread": 0.0,
  "rate": 3600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.2,
    0.5
  ],
  "speed": [
    0.215,
    0.645
  ],
  "size": [
    1.4333,
    1.4333
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.2188,
      0.4653
    ],
    [
      0.4531,
      0.6586
    ],
    [
      0.7031,
      0.8302
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
      0.1406,
      [
        0.2195,
        0.0055,
        0.0104,
        0.2123
      ]
    ],
    [
      0.2857,
      [
        0.2195,
        0.0055,
        0.0104,
        0.3585
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
    "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-2-0.effect"
  }
});
