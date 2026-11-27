import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.5.0.0.1 "LineColoe"
export default file.particles({
  "maxParticles": 1600,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 320
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    1.25,
    5.75
  ],
  "speedOverLife": [
    [
      0.0,
      1.8831
    ],
    [
      0.5156,
      0.9131
    ],
    [
      1.0,
      0.3637
    ]
  ],
  "size": [
    1.0,
    1.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.251
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.1002,
        0.0218,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.251
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.3278,
        0.1002,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
