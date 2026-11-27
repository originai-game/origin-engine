import { file } from '@origingame/engine/author';
// Effekseer node 3.0.5.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.8203,
    1.9922
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
    1.25,
    1.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0478,
        0.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5295,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
