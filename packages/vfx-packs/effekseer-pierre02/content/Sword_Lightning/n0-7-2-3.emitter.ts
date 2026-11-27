import { file } from '@origingame/engine/author';
// Effekseer node 0.7.2.3 "Burst3"
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
    27.0,
    27.0
  ],
  "size": [
    64.8,
    64.8
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
        0.7445,
        0.7969,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.7445,
        0.7969,
        1.0,
        0.251
      ]
    ],
    [
      0.2,
      [
        0.7445,
        0.7969,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.7969,
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
