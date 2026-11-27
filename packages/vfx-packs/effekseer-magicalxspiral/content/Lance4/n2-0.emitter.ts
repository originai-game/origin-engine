import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "Node"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "sphere",
    "radius": 1.5
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    30.0,
    30.0
  ],
  "size": [
    2.0,
    2.0
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      1.0,
      2.0
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
        0.0,
        0.7445,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.2195,
        0.0,
        0.7445,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0,
        0.7445,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2158,
        0.8671,
        0.502
      ]
    ],
    [
      0.6,
      [
        0.2195,
        0.2158,
        0.8671,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2158,
        0.8671,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line6"
  },
  "blend": "additive"
});
