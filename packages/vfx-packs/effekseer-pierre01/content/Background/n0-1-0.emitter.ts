import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "Node"
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
    14.3333,
    14.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1221.959,
    1221.959
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0002
    ],
    [
      0.125,
      0.0159
    ],
    [
      0.25,
      0.0629
    ],
    [
      0.375,
      0.141
    ],
    [
      0.5,
      0.2504
    ],
    [
      0.75,
      0.5628
    ],
    [
      0.875,
      0.7658
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.9651,
      [
        0.5356,
        0.7519,
        1.0,
        0.9222
      ]
    ],
    [
      1.0,
      [
        0.5356,
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
