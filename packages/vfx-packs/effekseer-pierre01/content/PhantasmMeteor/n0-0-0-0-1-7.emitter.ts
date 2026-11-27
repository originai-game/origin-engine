import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.7 "CosmosSmoke_White"
export default file.particles({
  "maxParticles": 169,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 9,
      "interval": 0.0667
    }
  ],
  "duration": 0.55,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    1.4434,
    5.7735
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
      0.1111,
      [
        1.0,
        0.9309,
        0.9829,
        0.1923
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.7424,
        0.9333,
        0.2042
      ]
    ],
    [
      0.7031,
      [
        1.0,
        0.4093,
        0.8295,
        0.1025
      ]
    ],
    [
      0.8594,
      [
        1.0,
        0.2977,
        0.7866,
        0.051
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
  "angularVelocity": [
    -120.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-1-7.effect"
  }
});
