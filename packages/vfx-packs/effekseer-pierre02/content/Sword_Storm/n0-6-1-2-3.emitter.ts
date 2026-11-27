import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.2.3 "Burst3"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 360.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    120.0,
    120.0
  ],
  "size": [
    288.0,
    288.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4444
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2222
    ],
    [
      1.0,
      0.5
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.2,
      [
        1.0,
        1.0,
        1.0,
        0.251
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
