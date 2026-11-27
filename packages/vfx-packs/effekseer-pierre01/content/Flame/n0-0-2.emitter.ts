import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Halo"
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
      "interval": 0.1
    }
  ],
  "duration": 0.6167,
  "looping": false,
  "lifetime": [
    1.3333,
    1.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.125,
      0.9634
    ],
    [
      0.4375,
      0.9305
    ],
    [
      0.625,
      0.8221
    ],
    [
      0.7344,
      0.6847
    ],
    [
      0.8281,
      0.507
    ],
    [
      0.9219,
      0.2625
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.8125,
      [
        1.0,
        0.5356,
        0.2195,
        0.2447
      ]
    ],
    [
      0.9062,
      [
        1.0,
        0.5356,
        0.2195,
        0.1365
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -30.0,
    30.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Halo"
  },
  "blend": "additive"
});
