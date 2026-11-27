import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0.0.3.0.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.0312,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 1200.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.041,
    0.2285
  ],
  "speedOverLife": [
    [
      0.0,
      7.3132
    ],
    [
      0.5156,
      0.0303
    ],
    [
      0.7031,
      2.3581
    ],
    [
      0.7344,
      3.7957
    ],
    [
      0.8438,
      4.8604
    ],
    [
      0.8594,
      1.7763
    ],
    [
      0.875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.2051,
    0.2637
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      0.4219,
      [
        0.1304,
        0.0284,
        1.0,
        0.502
      ]
    ],
    [
      0.8571,
      [
        0.0641,
        0.0139,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0104,
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
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      0.4219,
      [
        0.3838,
        0.1304,
        1.0,
        0.502
      ]
    ],
    [
      0.8571,
      [
        0.2555,
        0.0641,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
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
