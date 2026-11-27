import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.1.4 "Light"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 0.0,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.9167,
    1.0833
  ],
  "speed": [
    2.4,
    3.6
  ],
  "size": [
    4.0,
    4.0
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
        0.8994,
        0.8994,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.8994,
        0.8994,
        0.0471
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.8994,
        0.8994,
        0.0471
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8994,
        0.8994,
        0.0
      ]
    ]
  ],
  "gravity": [
    -1.5843,
    -9.4641,
    -4.9559
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
