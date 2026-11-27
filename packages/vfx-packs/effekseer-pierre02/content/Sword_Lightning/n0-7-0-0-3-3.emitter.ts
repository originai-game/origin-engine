import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.3 "Distortion"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.1833
    }
  ],
  "duration": 1.1167,
  "looping": false,
  "lifetime": [
    1.0,
    2.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.0,
    56.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1429
    ],
    [
      0.2188,
      0.389
    ],
    [
      0.4531,
      0.6098
    ],
    [
      0.7031,
      0.806
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.6275
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.6275
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -540.0,
    540.0
  ],
  "angularVelocity": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "alpha",
  "distortion": 0.01
});
