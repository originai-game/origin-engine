import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "Wave_Light"
export default file.particles({
  "maxParticles": 675,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 2.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    2.4,
    2.6
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
        1.0,
        0.8751,
        0.5605,
        0.0
      ]
    ],
    [
      0.08,
      [
        1.0,
        0.8751,
        0.5605,
        0.3765
      ]
    ],
    [
      0.8667,
      [
        1.0,
        0.8751,
        0.5605,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.5605,
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
