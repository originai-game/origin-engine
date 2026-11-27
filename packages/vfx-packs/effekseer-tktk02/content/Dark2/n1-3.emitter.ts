import { file } from '@origingame/engine/author';
// Effekseer node 1.3 "Glow1Core"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    15.0,
    15.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.7512
    ],
    [
      0.4531,
      0.5278
    ],
    [
      0.7031,
      0.3295
    ],
    [
      1.0,
      0.1333
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.0625,
      [
        0.865,
        0.1758,
        1.0,
        0.0941
      ]
    ],
    [
      0.125,
      [
        0.7515,
        0.1406,
        1.0,
        0.1882
      ]
    ],
    [
      0.25,
      [
        0.576,
        0.0898,
        1.0,
        0.3765
      ]
    ],
    [
      0.375,
      [
        0.453,
        0.0577,
        1.0,
        0.3765
      ]
    ],
    [
      0.5,
      [
        0.3671,
        0.0377,
        1.0,
        0.3765
      ]
    ],
    [
      0.7188,
      [
        0.2756,
        0.0195,
        1.0,
        0.2118
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle_Soft"
  },
  "blend": "additive"
});
