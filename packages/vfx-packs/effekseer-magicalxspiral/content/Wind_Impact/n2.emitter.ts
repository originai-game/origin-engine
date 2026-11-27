import { file } from '@origingame/engine/author';
// Effekseer node 2 "Line1"
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
    0.1667,
    0.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1667
    ],
    [
      1.0,
      0.1833
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.0478,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      0.8,
      [
        0.0478,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "rotation": [
    15.0,
    75.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave3"
  },
  "blend": "additive"
});
