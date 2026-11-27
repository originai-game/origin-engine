import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.10 "Stars"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.1875,
      2.1875,
      2.1875
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.0267,
  "looping": false,
  "lifetime": [
    1.1667,
    1.1667
  ],
  "speed": [
    0.065,
    0.2598
  ],
  "size": [
    0.0875,
    0.0875
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.4423,
        0.2195,
        0.5295,
        0.498
      ]
    ],
    [
      0.7143,
      [
        0.4423,
        0.2195,
        0.5295,
        0.498
      ]
    ],
    [
      1.0,
      [
        0.4423,
        0.2195,
        0.5295,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.687,
        0.5295,
        0.7445,
        0.749
      ]
    ],
    [
      0.7143,
      [
        0.687,
        0.5295,
        0.7445,
        0.749
      ]
    ],
    [
      1.0,
      [
        0.687,
        0.5295,
        0.7445,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
