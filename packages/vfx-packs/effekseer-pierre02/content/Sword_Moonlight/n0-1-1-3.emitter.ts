import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.3 "EmberVariation"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      1.2,
      0.25
    ]
  },
  "spread": 54.7356,
  "rate": 2.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1667,
    0.8333
  ],
  "speed": [
    2.0785,
    3.9799
  ],
  "size": [
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3
    ],
    [
      0.2188,
      0.501
    ],
    [
      0.4531,
      0.6814
    ],
    [
      0.7031,
      0.8416
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
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.0478,
        0.2845,
        0.2195,
        0.6255
      ]
    ],
    [
      0.3,
      [
        0.0478,
        0.2845,
        0.2195,
        0.6255
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -30.0,
    30.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "gravity": [
    -0.2112,
    -1.2619,
    -0.6608
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-1-1-3.effect"
  }
});
