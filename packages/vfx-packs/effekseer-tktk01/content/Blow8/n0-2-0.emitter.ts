import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0 "粒"
export default file.particles({
  "maxParticles": 338,
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
      "count": 30
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.25,
    0.4167
  ],
  "speed": [
    4.5,
    19.5
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
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
        0.0,
        0.0,
        0.0,
        0.8
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
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.8
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
