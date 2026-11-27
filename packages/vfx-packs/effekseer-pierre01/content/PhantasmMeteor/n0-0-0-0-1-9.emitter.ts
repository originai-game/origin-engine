import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.9 "CosmosSmoke_Green"
export default file.particles({
  "maxParticles": 113,
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
      "cycles": 6,
      "interval": 0.0167
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
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
      0.25,
      [
        0.0104,
        0.2845,
        0.2195,
        0.251
      ]
    ],
    [
      0.5,
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
      0.25,
      [
        0.1166,
        0.2845,
        0.2195,
        0.7529
      ]
    ],
    [
      0.5,
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
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-1-9.effect"
  }
});
