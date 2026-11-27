import { file } from '@origingame/engine/author';
// Effekseer node 10 "Bloom2"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.0,
    3.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.02,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      0.9,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle3"
  },
  "blend": "additive"
});
