import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.8.1 "Smoke_Variation2"
export default file.particles({
  "maxParticles": 57,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      1.9744,
      1.0497
    ]
  },
  "spread": 9.8779,
  "rate": 909.0909,
  "duration": 0.0495,
  "looping": false,
  "lifetime": [
    0.5,
    1.1667
  ],
  "speed": [
    6.0,
    42.0171
  ],
  "size": [
    19.55,
    19.55
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1023
    ],
    [
      0.5,
      0.5192
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
        0.9158,
        0.9574,
        0.0
      ]
    ],
    [
      0.3906,
      [
        1.0,
        0.9158,
        0.9574,
        0.0567
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.9158,
        0.9574,
        0.0627
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9158,
        0.9574,
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
    0.0,
    7.2,
    -4.32
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-8-1.effect"
  }
});
