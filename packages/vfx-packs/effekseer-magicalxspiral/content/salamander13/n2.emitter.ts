import { file } from '@origingame/engine/author';
// Effekseer node 2 "Node"
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
    1.6667,
    1.6667
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
        1.0,
        0.0,
        0.0,
        0.7529
      ]
    ],
    [
      0.9,
      [
        1.0,
        0.0,
        0.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle3"
  },
  "blend": "additive"
});
