import { file } from '@origingame/engine/author';
// Effekseer node 1 "グロー"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.3728,
      1.765,
      2.0
    ]
  },
  "spread": 79.3447,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0333
    }
  ],
  "duration": 0.4833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    8.4853,
    20.5913
  ],
  "speedOverLife": [
    [
      0.0,
      0.2582
    ],
    [
      0.125,
      0.0448
    ],
    [
      0.25,
      0.0091
    ],
    [
      0.375,
      0.1609
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      1.0269
    ],
    [
      0.75,
      1.7411
    ],
    [
      1.0,
      3.6952
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.1719,
      [
        0.3147,
        1.0,
        1.0,
        0.8208
      ]
    ],
    [
      0.3594,
      [
        0.4467,
        1.0,
        1.0,
        0.6158
      ]
    ],
    [
      0.7188,
      [
        0.7579,
        1.0,
        1.0,
        0.2377
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
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.6719,
      [
        1.0,
        1.0,
        1.0,
        0.2837
      ]
    ],
    [
      0.8438,
      [
        1.0,
        1.0,
        1.0,
        0.1229
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/t0002"
  },
  "blend": "additive"
});
