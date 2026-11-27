import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "Wave_Light"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 2.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    3.4,
    3.6
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.08
    ],
    [
      0.1562,
      0.1931
    ],
    [
      0.3281,
      0.341
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
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ],
    [
      0.0571,
      [
        0.2195,
        0.0055,
        0.0104,
        0.3765
      ]
    ],
    [
      0.9048,
      [
        0.2195,
        0.0055,
        0.0104,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "additive"
});
