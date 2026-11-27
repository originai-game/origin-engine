import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "Node"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    10.3923,
    10.3923
  ],
  "size": [
    0.5,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.5356,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.7445,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7445,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
