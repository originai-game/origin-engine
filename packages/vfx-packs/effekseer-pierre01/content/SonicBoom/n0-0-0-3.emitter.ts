import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "Particle"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      1.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.0,
    1.3333
  ],
  "speed": [
    6.7517,
    27.0067
  ],
  "size": [
    0.09,
    0.31
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    0.0,
    0.0,
    -54.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
