import { file } from '@origingame/engine/author';
// Effekseer node 0.0.15 "CosmosParticle"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.02,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.052,
    0.2078
  ],
  "size": [
    0.1,
    0.1
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.15,
      [
        0.7519,
        0.5356,
        1.0,
        0.6235
      ]
    ],
    [
      0.7,
      [
        0.7519,
        0.5356,
        1.0,
        0.6235
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
