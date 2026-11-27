import { file } from '@origingame/engine/author';
// Effekseer node 13 "Bloom"
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
    0.5833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.0,
    8.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.1429,
      [
        0.0478,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      0.8571,
      [
        0.0478,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
