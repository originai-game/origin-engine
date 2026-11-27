import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.6 "Blur"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.9294,
      1.1576
    ]
  },
  "spread": 28.0725,
  "rate": 300.0,
  "duration": 0.16,
  "looping": false,
  "lifetime": [
    1.0,
    2.0
  ],
  "speed": [
    10.6727,
    29.0328
  ],
  "speedOverLife": [
    [
      0.0,
      4.7561
    ],
    [
      0.1094,
      3.1314
    ],
    [
      0.2188,
      1.9366
    ],
    [
      0.2812,
      1.4781
    ],
    [
      0.3281,
      1.2412
    ],
    [
      0.4375,
      0.9869
    ],
    [
      0.4531,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    33.75,
    33.75
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6
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
        0.0
      ]
    ],
    [
      0.2222,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.5556,
      [
        1.0,
        1.0,
        1.0,
        0.3765
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.02
});
