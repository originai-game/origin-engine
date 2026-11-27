import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.1.0 "Smoke"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.15,
    0.6
  ],
  "size": [
    5.5,
    5.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0909
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
      0.64,
      [
        1.0,
        1.0,
        1.0,
        0.0784
      ]
    ],
    [
      0.8,
      [
        1.0,
        1.0,
        1.0,
        0.0784
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
  "gravity": [
    -1.5843,
    -9.4641,
    -4.9559
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Storm/n0-6-0-1-0.effect"
  }
});
