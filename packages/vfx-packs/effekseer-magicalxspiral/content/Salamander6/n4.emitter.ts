import { file } from '@origingame/engine/author';
// Effekseer node 4 "Bloom2"
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
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0,
        0.6275
      ]
    ],
    [
      0.8333,
      [
        1.0,
        0.2195,
        0.0,
        0.6275
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
