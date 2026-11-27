import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0.0.3.0.0.0.0 "LineMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.0312,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 1200.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.041,
    0.2285
  ],
  "speedOverLife": [
    [
      0.0,
      7.3132
    ],
    [
      0.5156,
      0.0303
    ],
    [
      0.7031,
      2.3581
    ],
    [
      0.7344,
      3.7957
    ],
    [
      0.8438,
      4.8604
    ],
    [
      0.8594,
      1.7763
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
    0.1172,
    0.1172
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.8571,
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
