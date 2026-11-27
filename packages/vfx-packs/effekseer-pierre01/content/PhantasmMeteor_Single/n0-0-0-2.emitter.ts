import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "Particle"
export default file.particles({
  "maxParticles": 39,
  "shape": {
    "kind": "box",
    "halfExtents": [
      7.5,
      7.5,
      7.5
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 36,
      "interval": 0.0333
    }
  ],
  "duration": 1.1833,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    3.2476,
    12.9904
  ],
  "size": [
    0.25,
    0.25
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
      0.4,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
