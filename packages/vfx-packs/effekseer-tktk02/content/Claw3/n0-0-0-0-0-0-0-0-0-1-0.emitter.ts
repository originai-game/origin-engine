import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0.0.0.0.1.0 "LineGlow"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.1162,
    0.1937
  ],
  "speedOverLife": [
    [
      0.0,
      5.1527
    ],
    [
      0.2031,
      2.1366
    ],
    [
      0.2969,
      1.1693
    ],
    [
      0.4062,
      0.4066
    ],
    [
      0.5469,
      0.0048
    ],
    [
      0.7031,
      0.322
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.226,
    0.2905
  ],
  "heightOverLife": [
    [
      0.0,
      0.625
    ],
    [
      1.0,
      0.625
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
        0.2195,
        0.0478,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.1166,
        0.0254,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0104,
        1.0,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.2195,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.3587,
        0.1166,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        1.0,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
