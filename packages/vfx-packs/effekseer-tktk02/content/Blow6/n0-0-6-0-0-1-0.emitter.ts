import { file } from '@origingame/engine/author';
// Effekseer node 0.0.6.0.0.1.0 "particleGlow"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1875,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    1.9688,
    4.7812
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
    3.75,
    3.75
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
      0.2,
      [
        1.0,
        0.0478,
        0.0104,
        0.0627
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0627
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.5356,
        0.1166,
        0.0627
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0627
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
