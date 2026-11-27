import { file } from '@origingame/engine/author';
// Effekseer node 5 "Bloom1"
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
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.8443
    ],
    [
      0.1562,
      0.7239
    ],
    [
      0.2344,
      0.6342
    ],
    [
      0.3281,
      0.5603
    ],
    [
      0.4688,
      0.5019
    ],
    [
      0.8281,
      0.463
    ],
    [
      1.0,
      0.4
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.3587,
        0.6275
      ]
    ],
    [
      0.8,
      [
        1.0,
        0.7519,
        0.3587,
        0.6275
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.3587,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
