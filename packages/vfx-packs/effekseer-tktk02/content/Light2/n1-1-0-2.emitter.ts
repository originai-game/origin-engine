import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.2 "LineGlow"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.3438,
    3.7812
  ],
  "speedOverLife": [
    [
      0.0,
      1.6037
    ],
    [
      0.5156,
      0.9527
    ],
    [
      1.0,
      0.5144
    ]
  ],
  "size": [
    4.5833,
    4.5833
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
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.4,
      [
        0.7445,
        1.0,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.4,
      [
        0.8671,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
