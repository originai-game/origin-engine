import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.7 "Burst3"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    30.0,
    30.0
  ],
  "size": [
    72.0,
    72.0
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
        0.9574,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.9574,
        0.7519,
        1.0,
        0.251
      ]
    ],
    [
      0.2,
      [
        0.9574,
        0.7519,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.9574,
        0.7519,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Burst_2"
  },
  "blend": "additive"
});
