import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.2 "FineMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.8438,
    1.4062
  ],
  "speedOverLife": [
    [
      0.0,
      1.4238
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      0.7016
    ]
  ],
  "size": [
    0.4688,
    0.6562
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0356,
        0.0078,
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0254,
        0.0055,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
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
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        0.1255
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
