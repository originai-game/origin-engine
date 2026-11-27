import { file } from '@origingame/engine/author';
// Effekseer node 11 "Particle"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0167
    }
  ],
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.25,
    0.4167
  ],
  "speed": [
    1.9486,
    7.7942
  ],
  "size": [
    0.25,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2233,
        0.0478,
        0.0
      ]
    ],
    [
      0.05,
      [
        1.0,
        0.2233,
        0.0478,
        0.7529
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2233,
        0.0478,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2233,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
