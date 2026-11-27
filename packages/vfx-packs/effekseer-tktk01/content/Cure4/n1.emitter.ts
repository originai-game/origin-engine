import { file } from '@origingame/engine/author';
// Effekseer node 1 "中心光"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.1719,
      0.6571
    ],
    [
      0.375,
      0.7845
    ],
    [
      0.5781,
      0.8691
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
        1.0,
        1.0,
        1.0,
        0.6863
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.6196
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
