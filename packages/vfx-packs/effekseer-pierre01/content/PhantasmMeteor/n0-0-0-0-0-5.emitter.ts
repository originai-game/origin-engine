import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.5 "CosmosSmoke"
export default file.particles({
  "maxParticles": 225,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.875,
      1.875,
      1.875
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 12,
      "interval": 0.0833
    }
  ],
  "duration": 0.9333,
  "looping": false,
  "lifetime": [
    1.5,
    1.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.5,
    2.5
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
      0.1667,
      [
        0.0104,
        0.2845,
        0.2195,
        0.251
      ]
    ],
    [
      0.6667,
      [
        0.0104,
        0.2845,
        0.2195,
        0.251
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
      0.1667,
      [
        0.1166,
        0.2845,
        0.2195,
        0.7529
      ]
    ],
    [
      0.6667,
      [
        0.1166,
        0.2845,
        0.2195,
        0.7529
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
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-0-5.effect"
  }
});
