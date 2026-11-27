import { file } from '@origingame/engine/author';
// Effekseer node 0.0.4 "Shockwave"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    181.0,
    181.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0442
    ],
    [
      0.125,
      0.2483
    ],
    [
      0.25,
      0.4282
    ],
    [
      0.375,
      0.5839
    ],
    [
      0.5,
      0.7155
    ],
    [
      0.625,
      0.8229
    ],
    [
      0.75,
      0.9061
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
        0.7519,
        0.5356,
        1.0,
        0.0941
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Shockwave"
  },
  "blend": "additive"
});
