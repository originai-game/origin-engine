import { file } from '@origingame/engine/author';
// Effekseer node 0 "Smoke"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 30
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    2.1213,
    8.4853
  ],
  "size": [
    1.0,
    0.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6562,
      0.0156
    ],
    [
      0.6719,
      0.0078
    ],
    [
      1.0,
      0.5
    ]
  ],
  "colorOverLife": [
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
      0.1094,
      [
        0.7838,
        0.7838,
        0.7838,
        0.7835
      ]
    ],
    [
      0.2188,
      [
        0.6075,
        0.6075,
        0.6075,
        0.6011
      ]
    ],
    [
      0.4531,
      [
        0.319,
        0.319,
        0.319,
        0.3096
      ]
    ],
    [
      0.5938,
      [
        0.1878,
        0.1878,
        0.1878,
        0.1893
      ]
    ],
    [
      0.75,
      [
        0.0761,
        0.0761,
        0.0761,
        0.0934
      ]
    ],
    [
      0.875,
      [
        0.0192,
        0.0192,
        0.0192,
        0.0393
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -70.0,
    70.0
  ],
  "angularVelocity": [
    -3.0,
    3.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/smoke_tex"
  },
  "blend": "alpha"
});
