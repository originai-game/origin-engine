import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0.4 "EmberVariation"
export default file.particles({
  "maxParticles": 180,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.6,
      1.0,
      0.6
    ]
  },
  "spread": 20.6627,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 72
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.05,
    0.65
  ],
  "speed": [
    27.0749,
    43.2896
  ],
  "speedOverLife": [
    [
      0.0,
      1.192
    ],
    [
      1.0,
      0.8308
    ]
  ],
  "size": [
    1.5833,
    1.5833
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
      0.1406,
      [
        0.2195,
        0.0055,
        0.0104,
        0.1592
      ]
    ],
    [
      0.2857,
      [
        0.2195,
        0.0055,
        0.0104,
        0.2689
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
    -30.0,
    30.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/LossOfBlood/n0-1-1-0-4.effect"
  }
});
