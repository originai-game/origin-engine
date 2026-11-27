import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.5.0.0.0 "LineMain"
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
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        1.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.3621,
        0.1406,
        1.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.2468,
        0.0898,
        1.0
      ]
    ],
    [
      0.4062,
      [
        1.0,
        0.1568,
        0.0518,
        1.0
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.0954,
        0.0274,
        1.0
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
