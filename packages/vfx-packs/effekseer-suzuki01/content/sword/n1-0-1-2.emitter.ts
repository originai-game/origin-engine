import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.2 "Splash"
export default file.particles({
  "maxParticles": 750,
  "shape": {
    "kind": "point"
  },
  "spread": 81.9505,
  "rate": 6000.0,
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    42.8486,
    68.6731
  ],
  "size": [
    0.5,
    0.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2158,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2158,
        0.0,
        1.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -360.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
