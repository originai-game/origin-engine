import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Distortion"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.7129
    ],
    [
      0.4531,
      0.4552
    ],
    [
      0.7031,
      0.2263
    ],
    [
      1.0,
      0.0
    ]
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
      0.25,
      [
        1.0,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      0.5,
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion"
  },
  "blend": "additive",
  "distortion": 0.004
});
