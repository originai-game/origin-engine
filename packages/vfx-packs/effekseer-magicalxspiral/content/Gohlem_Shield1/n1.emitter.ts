import { file } from '@origingame/engine/author';
// Effekseer node 1 "Node"
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
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2
    ],
    [
      1.0,
      0.2
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.7519,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.5356,
        0.7519,
        0.6275
      ]
    ],
    [
      0.9,
      [
        1.0,
        0.5356,
        0.7519,
        0.6275
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
