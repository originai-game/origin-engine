import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.55,
    6.45
  ],
  "speedOverLife": [
    [
      0.0,
      0.5667
    ],
    [
      0.4219,
      1.2389
    ],
    [
      0.8125,
      1.6942
    ],
    [
      0.8281,
      1.4231
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
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
        0.0104,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.147,
        0.007,
        1.0,
        1.0
      ]
    ],
    [
      0.8333,
      [
        0.0671,
        0.0032,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0023,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4134,
        0.032,
        1.0,
        1.0
      ]
    ],
    [
      0.8333,
      [
        0.2618,
        0.0146,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
