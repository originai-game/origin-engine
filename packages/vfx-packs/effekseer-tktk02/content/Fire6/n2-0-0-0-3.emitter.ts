import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.3 "Distortion"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
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
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
