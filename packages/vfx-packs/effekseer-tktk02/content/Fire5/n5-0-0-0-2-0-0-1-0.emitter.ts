import { file } from '@origingame/engine/author';
// Effekseer node 5.0.0.0.2.0.0.1.0 "particleColor"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.8357,
    2.3643
  ],
  "speedOverLife": [
    [
      0.0,
      4.0533
    ],
    [
      0.2344,
      1.8524
    ],
    [
      0.3438,
      1.108
    ],
    [
      0.4688,
      0.4938
    ],
    [
      0.6719,
      0.034
    ],
    [
      0.8438,
      0.1654
    ],
    [
      0.875,
      0.0185
    ],
    [
      1.0,
      0.1063
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
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.034,
        0.0074,
        0.251
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.0281,
        0.0061,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.1564,
        0.034,
        0.251
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.1291,
        0.0281,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
