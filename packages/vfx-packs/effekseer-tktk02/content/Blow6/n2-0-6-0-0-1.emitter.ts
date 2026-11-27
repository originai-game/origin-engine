import { file } from '@origingame/engine/author';
// Effekseer node 2.0.6.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.9375,
    2.8125
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
      0.5781,
      0.6756
    ],
    [
      0.7812,
      0.3546
    ],
    [
      0.7969,
      0.2383
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
      0.2,
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
      0.2,
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
