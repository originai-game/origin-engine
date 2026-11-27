import { file } from '@origingame/engine/author';
// Effekseer node 0.0.10 "CosmosSmokeVariation"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.7,
      0.7,
      0.7
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.05
    }
  ],
  "duration": 0.1167,
  "looping": false,
  "lifetime": [
    1.6,
    1.6
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.5,
    1.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0104,
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.2083,
      [
        0.0104,
        0.2845,
        0.2195,
        0.0627
      ]
    ],
    [
      0.6875,
      [
        0.0104,
        0.2845,
        0.2195,
        0.0627
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.2845,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.2083,
      [
        0.1166,
        0.2845,
        0.2195,
        0.4392
      ]
    ],
    [
      0.6875,
      [
        0.1166,
        0.2845,
        0.2195,
        0.4392
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.2845,
        0.2195,
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
    "$ogRef": "base/vfx/effekseer-pierre01/Herald/n0-0-10.effect"
  }
});
