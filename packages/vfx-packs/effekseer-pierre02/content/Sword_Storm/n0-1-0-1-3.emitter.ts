import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.3 "Particle"
export default file.particles({
  "maxParticles": 24,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.5,
      0.5,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.0,
    1.3333
  ],
  "speed": [
    1.5075,
    6.0299
  ],
  "size": [
    0.1,
    0.1
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
      0.4286,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    -1800.0,
    1800.0
  ],
  "gravity": [
    -1.5843,
    -9.4641,
    -4.9559
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Hard"
  },
  "blend": "additive"
});
