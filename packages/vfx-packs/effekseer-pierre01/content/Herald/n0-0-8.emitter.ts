import { file } from '@origingame/engine/author';
// Effekseer node 0.0.8 "SmokeVariation"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.1333
    }
  ],
  "duration": 1.2167,
  "looping": false,
  "lifetime": [
    0.4,
    0.4
  ],
  "speed": [
    1.299,
    5.1962
  ],
  "size": [
    2.576,
    3.864
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7764
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
      0.3333,
      [
        1.0,
        0.6364,
        0.903,
        0.2276
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.4821,
        0.8547,
        0.2154
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.3122,
        0.7925,
        0.0996
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/Herald/n0-0-8.effect"
  }
});
