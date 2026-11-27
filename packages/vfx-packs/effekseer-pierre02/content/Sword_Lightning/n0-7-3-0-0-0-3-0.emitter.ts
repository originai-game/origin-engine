import { file } from '@origingame/engine/author';
// Effekseer node 0.7.3.0.0.0.3.0 "Ray"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.1245,
      0.0346
    ]
  },
  "spread": 0.5286,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    28.1475,
    45.0141
  ],
  "speedOverLife": [
    [
      0.0,
      0.899
    ],
    [
      0.2656,
      1.3818
    ],
    [
      0.3906,
      1.4445
    ],
    [
      0.5312,
      1.3783
    ],
    [
      0.7812,
      0.9294
    ],
    [
      0.7969,
      0.6254
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    36.0,
    72.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6719,
      0.6419
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
        0.0461,
        0.061,
        0.5356,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.0461,
        0.061,
        0.5356,
        0.502
      ]
    ],
    [
      0.3333,
      [
        0.0461,
        0.061,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0461,
        0.061,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
