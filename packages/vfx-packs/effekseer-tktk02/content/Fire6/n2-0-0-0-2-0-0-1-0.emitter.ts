import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.2.0.0.1.0 "particleColor"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.75,
    1.25
  ],
  "speedOverLife": [
    [
      0.0,
      4.3039
    ],
    [
      0.2344,
      1.8585
    ],
    [
      0.3438,
      1.052
    ],
    [
      0.4531,
      0.4735
    ],
    [
      0.625,
      0.025
    ],
    [
      0.8438,
      0.2683
    ],
    [
      0.875,
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
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.034,
        0.0074,
        0.251
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.0281,
        0.0061,
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
      0.2857,
      [
        1.0,
        0.1564,
        0.034,
        0.251
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.1291,
        0.0281,
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
