import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "Dark"
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    3.0,
    3.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    24.75,
    24.75
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6364
    ],
    [
      0.4531,
      0.8345
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
        0.0005,
        0.0012,
        0.0055,
        0.0
      ]
    ],
    [
      0.0833,
      [
        0.0005,
        0.0012,
        0.0055,
        0.251
      ]
    ],
    [
      0.7778,
      [
        0.0005,
        0.0012,
        0.0055,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0005,
        0.0012,
        0.0055,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-0-0-0.effect"
  }
});
