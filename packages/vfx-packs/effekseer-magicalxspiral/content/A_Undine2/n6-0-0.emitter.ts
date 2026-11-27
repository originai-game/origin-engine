import { file } from '@origingame/engine/author';
// Effekseer node 6.0.0 "Node"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    6.0,
    6.0
  ],
  "size": [
    0.25,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1636,
        0.6388,
        1.0,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.1636,
        0.6388,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.1636,
        0.6388,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2845,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.2845,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2845,
        0.8751,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -18.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
