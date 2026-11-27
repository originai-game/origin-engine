import { file } from '@origingame/engine/author';
// Effekseer node 5.0.0.0.2.0.0.0 "particleMain"
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
    0.4179,
    1.1821
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
    0.25,
    0.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.4286,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
