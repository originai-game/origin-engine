import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.4.0 "Smoke"
export default file.particles({
  "maxParticles": 2250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      1.5,
      0.001
    ]
  },
  "spread": 20.6626,
  "rate": 3000.0,
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.5,
    1.0
  ],
  "speed": [
    12.0234,
    19.224
  ],
  "size": [
    8.25,
    8.25
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7273
    ],
    [
      1.0,
      1.0
    ]
  ],
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
      0.0625,
      [
        1.0,
        1.0,
        1.0,
        0.0256
      ]
    ],
    [
      0.2,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.5333,
      [
        1.0,
        1.0,
        1.0,
        0.1255
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
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -300.0,
    300.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Storm/n0-7-0-4-0.effect"
  }
});
