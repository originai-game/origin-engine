import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "Debris Trail"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1167,
    0.1167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.8355,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      0.1429,
      [
        0.8355,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      0.7188,
      [
        0.8355,
        0.0,
        0.0,
        0.1103
      ]
    ],
    [
      0.8594,
      [
        0.8355,
        0.0,
        0.0,
        0.0402
      ]
    ],
    [
      1.0,
      [
        0.8355,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9158,
        0.586,
        0.0278,
        0.749
      ]
    ],
    [
      0.1429,
      [
        0.9158,
        0.586,
        0.0278,
        0.749
      ]
    ],
    [
      0.7188,
      [
        0.9158,
        0.586,
        0.0278,
        0.1646
      ]
    ],
    [
      0.8594,
      [
        0.9158,
        0.586,
        0.0278,
        0.06
      ]
    ],
    [
      1.0,
      [
        0.9158,
        0.586,
        0.0278,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/smoke_tex"
  },
  "blend": "additive"
});
