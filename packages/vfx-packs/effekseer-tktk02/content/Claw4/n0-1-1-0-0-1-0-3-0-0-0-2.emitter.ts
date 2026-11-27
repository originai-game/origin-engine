import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0.0.1.0.3.0.0.0.2 "particleGlow"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5833,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 1920.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.875,
    4.375
  ],
  "speedOverLife": [
    [
      0.0,
      2.6617
    ],
    [
      0.2031,
      1.7921
    ],
    [
      0.3906,
      1.1489
    ],
    [
      0.5938,
      0.6438
    ],
    [
      0.7812,
      0.3546
    ],
    [
      0.7969,
      0.2384
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    5.8333,
    5.8333
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.2195,
        0.0,
        0.0
      ]
    ],
    [
      0.4,
      [
        0.0,
        0.2195,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.2195,
        0.0,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.5295,
        0.2158,
        0.0
      ]
    ],
    [
      0.4,
      [
        0.2158,
        0.5295,
        0.2158,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.5295,
        0.2158,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
