import { file } from '@origingame/engine/author';
// Effekseer node 7 "Particle1"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.1333,
    0.2
  ],
  "speed": [
    7.7942,
    31.1769
  ],
  "size": [
    0.3,
    0.9
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.2195,
        0.1166,
        0.502
      ]
    ],
    [
      0.5,
      [
        0.5356,
        0.2195,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.2195,
        0.1166,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.7519,
        0.502
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2195,
        0.7519,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
