import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0.0.0.0.0.0 "Distortion"
export default file.particles({
  "maxParticles": 439,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3889,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.8333
  ],
  "speed": [
    1.1667,
    3.5
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
    0.3889,
    0.3889
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.0,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      0.4286,
      [
        1.0,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion5"
  },
  "blend": "additive",
  "distortion": 0.004
});
