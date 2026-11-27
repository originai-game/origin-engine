import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0 "glow"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.75,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.15,
    3.45
  ],
  "speedOverLife": [
    [
      0.0,
      8.361
    ],
    [
      0.0625,
      5.5279
    ],
    [
      0.1094,
      3.8397
    ],
    [
      0.1406,
      3.0007
    ],
    [
      0.1719,
      2.3906
    ],
    [
      0.2344,
      1.8577
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
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
