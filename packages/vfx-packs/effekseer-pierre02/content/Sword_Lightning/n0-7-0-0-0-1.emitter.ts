import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.1 "Stars"
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
    4.6667,
    4.6667
  ],
  "speed": [
    0.2598,
    1.0392
  ],
  "size": [
    0.75,
    1.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.3736,
        0.5295,
        0.0
      ]
    ],
    [
      0.0643,
      [
        0.2195,
        0.3736,
        0.5295,
        0.6275
      ]
    ],
    [
      0.7857,
      [
        0.2195,
        0.3736,
        0.5295,
        0.6275
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.3736,
        0.5295,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.6456,
        0.7445,
        0.0
      ]
    ],
    [
      0.0643,
      [
        0.5295,
        0.6456,
        0.7445,
        0.8118
      ]
    ],
    [
      0.7857,
      [
        0.5295,
        0.6456,
        0.7445,
        0.8118
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.6456,
        0.7445,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Hard"
  },
  "blend": "additive"
});
