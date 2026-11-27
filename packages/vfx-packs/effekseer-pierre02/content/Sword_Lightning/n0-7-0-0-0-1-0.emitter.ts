import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.1.0 "Ray"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      48.0,
      4.0,
      48.0
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.0267,
  "looping": false,
  "lifetime": [
    4.5,
    4.5
  ],
  "speed": [
    0.2598,
    1.0392
  ],
  "size": [
    4.0,
    4.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.2428,
        0.5295,
        0.0
      ]
    ],
    [
      0.0667,
      [
        0.1166,
        0.2428,
        0.5295,
        0.251
      ]
    ],
    [
      0.7778,
      [
        0.1166,
        0.2428,
        0.5295,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.2428,
        0.5295,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        0.7667,
        0.7445,
        0.0
      ]
    ],
    [
      0.0667,
      [
        0.7445,
        0.7667,
        0.7445,
        0.7529
      ]
    ],
    [
      0.7778,
      [
        0.7445,
        0.7667,
        0.7445,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.7667,
        0.7445,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
