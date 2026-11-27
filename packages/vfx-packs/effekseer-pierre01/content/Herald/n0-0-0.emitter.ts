import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "Light1"
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
    10.7919,
    10.7919
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0463
    ],
    [
      0.125,
      0.3605
    ],
    [
      0.25,
      0.612
    ],
    [
      0.375,
      0.8011
    ],
    [
      0.5,
      0.9275
    ],
    [
      0.75,
      0.9929
    ],
    [
      0.875,
      0.9317
    ],
    [
      1.0,
      0.808
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.0417,
      [
        0.8751,
        0.7519,
        1.0,
        0.251
      ]
    ],
    [
      0.3333,
      [
        0.8751,
        0.7519,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
