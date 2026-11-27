import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3.0.4 "EmberVariation"
export default file.particles({
  "maxParticles": 130,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.425,
      1.425,
      1.425
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 52,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.05,
    0.65
  ],
  "speed": [
    4.275,
    12.825
  ],
  "size": [
    1.7812,
    1.7812
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1406,
      [
        1.0,
        1.0,
        1.0,
        0.1592
      ]
    ],
    [
      0.2857,
      [
        1.0,
        1.0,
        1.0,
        0.2689
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
    "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-4.effect"
  }
});
