import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.2.0.1 "EmberVariation"
export default file.particles({
  "maxParticles": 135,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.9,
      1.9,
      1.9
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 27,
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
    17.1
  ],
  "size": [
    4.75,
    4.75
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
        1.0,
        0.1166,
        0.0104,
        0.0
      ]
    ],
    [
      0.1935,
      [
        1.0,
        0.1166,
        0.0104,
        0.502
      ]
    ],
    [
      0.3226,
      [
        1.0,
        0.1166,
        0.0104,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
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
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-2-0-1.effect"
  }
});
