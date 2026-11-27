import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.2.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.4125,
    1.1125
  ],
  "speedOverLife": [
    [
      0.0,
      3.8929
    ],
    [
      0.2344,
      1.8429
    ],
    [
      0.3594,
      1.0558
    ],
    [
      0.4688,
      0.5546
    ],
    [
      0.6562,
      0.1025
    ],
    [
      0.8438,
      0.1645
    ],
    [
      0.875,
      0.0012
    ],
    [
      1.0,
      0.015
    ]
  ],
  "size": [
    0.4375,
    0.5625
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.1564,
        0.034,
        1.0
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.1291,
        0.0281,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.4298,
        0.1564,
        1.0
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.3816,
        0.1291,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/FireFog"
  },
  "blend": "additive"
});
