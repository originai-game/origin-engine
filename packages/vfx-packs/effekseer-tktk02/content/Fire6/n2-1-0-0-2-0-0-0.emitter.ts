import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.0.2.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.375,
    0.625
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
    0.25,
    0.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.2857,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.4286,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
