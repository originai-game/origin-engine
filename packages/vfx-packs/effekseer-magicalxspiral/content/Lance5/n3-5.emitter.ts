import { file } from '@origingame/engine/author';
// Effekseer node 3.5 "Bloom"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.05,
      [
        0.2195,
        0.2195,
        0.2195,
        0.251
      ]
    ],
    [
      0.8,
      [
        0.2195,
        0.2195,
        0.2195,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
