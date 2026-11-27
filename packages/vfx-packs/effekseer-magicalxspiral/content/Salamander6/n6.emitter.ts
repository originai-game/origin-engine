import { file } from '@origingame/engine/author';
// Effekseer node 6 "Stars"
export default file.particles({
  "maxParticles": 39,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    1.7321,
    6.9282
  ],
  "size": [
    0.0,
    0.6
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.6388,
        0.5356,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.5356,
        0.6388,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.6388,
        0.5356,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.8751,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
