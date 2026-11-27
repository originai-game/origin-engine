import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "冷気"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 29999.998,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.0833,
    0.4167
  ],
  "speed": [
    13.0,
    31.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1267
    ],
    [
      0.25,
      1.2147
    ],
    [
      0.375,
      0.9107
    ],
    [
      0.5,
      0.7139
    ],
    [
      0.625,
      0.6245
    ],
    [
      0.75,
      0.6424
    ],
    [
      1.0,
      0.9911
    ]
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1667
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.3112,
        1.0,
        1.0,
        0.0784
      ]
    ],
    [
      0.2,
      [
        0.3112,
        1.0,
        1.0,
        0.0863
      ]
    ],
    [
      1.0,
      [
        0.3112,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3001"
  },
  "blend": "additive"
});
