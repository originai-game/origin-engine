import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 3000.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.3,
    4.5
  ],
  "speedOverLife": [
    [
      0.0,
      2.9785
    ],
    [
      0.1562,
      2.1316
    ],
    [
      0.2969,
      1.4897
    ],
    [
      0.5,
      0.7996
    ],
    [
      0.7031,
      0.3896
    ],
    [
      0.7188,
      0.0808
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.875,
    1.125
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.1047,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.4286,
      [
        0.0641,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.7031,
      [
        0.0152,
        0.0,
        0.0,
        0.5195
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
        0.5295,
        0.2158,
        0.2158,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.4244,
        0.2168,
        0.2168,
        1.0
      ]
    ],
    [
      0.4286,
      [
        0.3765,
        0.2174,
        0.2174,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
