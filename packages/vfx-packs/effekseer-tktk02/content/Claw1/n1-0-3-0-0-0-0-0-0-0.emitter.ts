import { file } from '@origingame/engine/author';
// Effekseer node 1.0.3.0.0.0.0.0.0.0 "Distortion"
export default file.particles({
  "maxParticles": 314,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3333,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.5
  ],
  "speed": [
    1.0,
    3.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    0.3333,
    0.3333
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.0379,
        0.0379,
        0.0379,
        0.0627
      ]
    ],
    [
      0.4,
      [
        0.0292,
        0.0292,
        0.0292,
        0.0941
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0104,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3537,
        0.3537,
        0.3537,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.2674,
        0.2674,
        0.2674,
        0.1255
      ]
    ],
    [
      0.4,
      [
        0.194,
        0.194,
        0.194,
        0.1882
      ]
    ],
    [
      0.7031,
      [
        0.1067,
        0.1067,
        0.1067,
        0.0931
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        0.0478,
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion2"
  },
  "blend": "additive",
  "distortion": 0.04
});
