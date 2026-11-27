import { file } from '@origingame/engine/author';
// Effekseer node 0.1.6 "DarkEye"
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
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    2.0,
    2.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.7206,
    1.7206
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2906
    ],
    [
      0.1875,
      0.7908
    ],
    [
      0.3906,
      1.0006
    ],
    [
      0.5156,
      0.9581
    ],
    [
      0.6406,
      0.7848
    ],
    [
      0.75,
      0.526
    ],
    [
      0.9062,
      0.0175
    ],
    [
      1.0,
      0.4417
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
      0.125,
      [
        1.0,
        0.8751,
        0.7519,
        1.0
      ]
    ],
    [
      0.9167,
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
