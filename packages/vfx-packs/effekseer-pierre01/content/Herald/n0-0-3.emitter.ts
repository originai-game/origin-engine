import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "DarkEye"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.05
    }
  ],
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    1.7,
    1.7
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.424,
    4.424
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7233
    ],
    [
      0.125,
      0.8476
    ],
    [
      0.25,
      0.9352
    ],
    [
      0.375,
      0.986
    ],
    [
      0.5,
      1.0
    ],
    [
      0.75,
      0.9179
    ],
    [
      0.875,
      0.8217
    ],
    [
      1.0,
      0.6887
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ],
    [
      0.2941,
      [
        1.0,
        0.8751,
        0.7519,
        0.1882
      ]
    ],
    [
      0.8039,
      [
        1.0,
        0.8751,
        0.7519,
        0.1882
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -300.0,
    300.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "subtractive"
});
