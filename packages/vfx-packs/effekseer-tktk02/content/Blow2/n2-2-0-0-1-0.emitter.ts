import { file } from '@origingame/engine/author';
// Effekseer node 2.2.0.0.1.0 "particleGlow"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    4.8,
    16.8
  ],
  "speedOverLife": [
    [
      0.0,
      3.4002
    ],
    [
      0.1406,
      2.4105
    ],
    [
      0.2656,
      1.6714
    ],
    [
      0.4375,
      0.9138
    ],
    [
      0.6094,
      0.4556
    ],
    [
      0.6406,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    5.0,
    5.0
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
      0.25,
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
      0.25,
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
