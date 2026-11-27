import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0 "火メイン"
export default file.particles({
  "maxParticles": 2250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 1000.0,
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.6508,
    0.6508
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.9787
    ],
    [
      0.1562,
      0.9516
    ],
    [
      0.2344,
      0.9195
    ],
    [
      0.3281,
      0.8755
    ],
    [
      0.4688,
      0.8009
    ],
    [
      0.8281,
      0.5925
    ],
    [
      1.0,
      0.5
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      1.1708
    ],
    [
      0.1562,
      1.3875
    ],
    [
      0.2344,
      1.6442
    ],
    [
      0.3281,
      1.9963
    ],
    [
      0.4688,
      2.5926
    ],
    [
      0.8281,
      4.2603
    ],
    [
      1.0,
      5.0
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
        0.0143,
        0.3345,
        0.3587,
        0.0
      ]
    ],
    [
      0.0667,
      [
        0.0162,
        0.3634,
        0.3865,
        0.1801
      ]
    ],
    [
      0.4889,
      [
        0.0278,
        0.5255,
        0.5411,
        0.1978
      ]
    ],
    [
      1.0,
      [
        0.043,
        0.7227,
        0.7263,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/smoke_tex"
  },
  "blend": "subtractive"
});
