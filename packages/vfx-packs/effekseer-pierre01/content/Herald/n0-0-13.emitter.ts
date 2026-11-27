import { file } from '@origingame/engine/author';
// Effekseer node 0.0.13 "DarkEye"
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
    1.8333,
    1.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.7242,
    1.7242
  ],
  "sizeOverLife": [
    [
      0.0,
      0.29
    ],
    [
      0.2344,
      0.8374
    ],
    [
      0.3594,
      0.9717
    ],
    [
      0.4844,
      0.9964
    ],
    [
      0.7031,
      0.7757
    ],
    [
      0.8438,
      0.4565
    ],
    [
      0.9844,
      0.0014
    ],
    [
      1.0,
      0.0609
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
      0.1364,
      [
        1.0,
        0.8751,
        0.7519,
        1.0
      ]
    ],
    [
      0.8182,
      [
        1.0,
        0.8751,
        0.7519,
        1.0
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
