import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.3 "Distortion"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.8438,
    1.4062
  ],
  "speedOverLife": [
    [
      0.0,
      1.4238
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      0.7016
    ]
  ],
  "size": [
    0.9375,
    1.3125
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.8086
    ],
    [
      0.4531,
      0.6368
    ],
    [
      0.7031,
      0.4842
    ],
    [
      1.0,
      0.3333
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
        0.251
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion10"
  },
  "blend": "additive",
  "distortion": 0.004
});
