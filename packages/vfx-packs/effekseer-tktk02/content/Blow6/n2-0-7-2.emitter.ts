import { file } from '@origingame/engine/author';
// Effekseer node 2.0.7.2 "LineGlow"
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
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    15.625,
    15.625
  ],
  "heightOverLife": [
    [
      0.0,
      0.3
    ],
    [
      1.0,
      0.3
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.05,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ],
    [
      0.05,
      [
        1.0,
        0.5356,
        0.1166,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
