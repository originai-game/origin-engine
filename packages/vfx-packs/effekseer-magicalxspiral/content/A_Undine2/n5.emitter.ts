import { file } from '@origingame/engine/author';
// Effekseer node 5 "Bloom2"
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
    0.4167,
    0.4167
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
        0.2195,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.04,
      [
        0.2195,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.2195,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle3"
  },
  "blend": "additive"
});
