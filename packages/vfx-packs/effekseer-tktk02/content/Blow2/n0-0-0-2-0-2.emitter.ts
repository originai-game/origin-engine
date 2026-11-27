import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0.2 "particleGlow"
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
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    2.4,
    4.8
  ],
  "speedOverLife": [
    [
      0.0,
      8.385
    ],
    [
      0.0156,
      7.8205
    ],
    [
      0.0625,
      5.7751
    ],
    [
      0.1094,
      4.0776
    ],
    [
      0.1719,
      2.3557
    ],
    [
      0.2344,
      1.2525
    ],
    [
      0.2656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.5,
    2.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.1166,
        0.0254,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3587,
        0.0781,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.3587,
        0.0781,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3587,
        0.0781,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
