import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2.0.0 "EmberVariation"
export default file.particles({
  "maxParticles": 180,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.4,
      0.2
    ]
  },
  "spread": 109.4712,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 36,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.2167,
    0.8167
  ],
  "speed": [
    5.7,
    9.832
  ],
  "speedOverLife": [
    [
      0.0,
      0.5376
    ],
    [
      0.5,
      1.0407
    ],
    [
      1.0,
      1.3034
    ]
  ],
  "size": [
    3.1667,
    3.1667
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      0.2188,
      0.5723
    ],
    [
      0.4531,
      0.7269
    ],
    [
      0.7031,
      0.8642
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
      0.1935,
      [
        0.2195,
        0.0055,
        0.0104,
        0.502
      ]
    ],
    [
      0.3226,
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
    "$ogRef": "base/vfx/effekseer-pierre02/LossOfBlood/n0-1-2-0-0.effect"
  }
});
