import { file } from '@origingame/engine/author';
// Effekseer node 1.1.3.0.1 "glow大"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.375,
    4.125
  ],
  "speedOverLife": [
    [
      0.0,
      2.1325
    ],
    [
      0.3281,
      1.4605
    ],
    [
      0.6562,
      1.0585
    ],
    [
      0.6719,
      0.1752
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
        0.1166,
        0.0,
        0.0104,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.0,
        0.1166,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.0,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
