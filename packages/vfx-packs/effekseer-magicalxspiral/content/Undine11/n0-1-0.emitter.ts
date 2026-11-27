import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "Node"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "sphere",
    "radius": 0.5
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
    5.1962,
    5.1962
  ],
  "size": [
    0.25,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0104,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.0104,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.1166,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
