import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0 "Fog"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.0938,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.9844,
    2.3906
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
    0.75,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.1562,
      [
        0.122,
        0.4205,
        1.0,
        0.5208
      ]
    ],
    [
      0.3,
      [
        0.0589,
        0.3278,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.0104,
        0.2195,
        1.0,
        0.7143
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.1166,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/fog2"
  },
  "blend": "additive"
});
