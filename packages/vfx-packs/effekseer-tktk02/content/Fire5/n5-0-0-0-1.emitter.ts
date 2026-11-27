import { file } from '@origingame/engine/author';
// Effekseer node 5.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 188,
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0857,
    1.1143
  ],
  "speedOverLife": [
    [
      0.0,
      4.2171
    ],
    [
      0.2344,
      1.8602
    ],
    [
      0.4531,
      0.5031
    ],
    [
      0.6406,
      0.0221
    ],
    [
      0.75,
      0.0323
    ],
    [
      0.8594,
      0.2569
    ],
    [
      0.8906,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.875,
    1.125
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
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
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
      0.25,
      [
        1.0,
        0.4423,
        0.1636,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.3587,
        0.1166,
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
