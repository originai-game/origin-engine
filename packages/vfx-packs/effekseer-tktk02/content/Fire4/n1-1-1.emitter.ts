import { file } from '@origingame/engine/author';
// Effekseer node 1.1.1 "Distortion"
export default file.particles({
  "maxParticles": 77,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0781,
      0.2636
    ],
    [
      0.1719,
      0.5118
    ],
    [
      0.2656,
      0.6949
    ],
    [
      0.375,
      0.8395
    ],
    [
      0.4688,
      0.9156
    ],
    [
      0.5781,
      0.9625
    ],
    [
      1.0,
      1.0
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
      0.1667,
      [
        1.0,
        1.0,
        0.0,
        0.502
      ]
    ],
    [
      0.6667,
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.004
});
