import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1 "glow"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1000
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.12,
    2.76
  ],
  "speedOverLife": [
    [
      0.0,
      3.019
    ],
    [
      0.25,
      1.6954
    ],
    [
      0.5,
      0.7391
    ],
    [
      0.75,
      0.1742
    ],
    [
      1.0,
      0.0003
    ]
  ],
  "size": [
    0.4,
    0.4
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.3112,
        0.0,
        0.0478,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.3112,
        0.0,
        0.0478,
        0.5882
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9574,
        0.0,
        0.2195,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.9574,
        0.0,
        0.2195,
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
