import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.1.0 "particleColor"
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
  "rate": 600.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    1.65,
    3.45
  ],
  "speedOverLife": [
    [
      0.0,
      5.5153
    ],
    [
      0.2031,
      2.1086
    ],
    [
      0.2969,
      1.0615
    ],
    [
      0.3906,
      0.3703
    ],
    [
      0.5156,
      0.0025
    ],
    [
      0.6562,
      0.3451
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.75,
    2.25
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
      0.1667,
      [
        1.0,
        0.0395,
        0.0086,
        0.251
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.0196,
        0.0043,
        0.251
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
      0.1667,
      [
        1.0,
        0.1813,
        0.0395,
        0.251
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.09,
        0.0196,
        0.251
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
