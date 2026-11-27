import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "一段目衝撃"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 50
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1333,
    0.3667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0278,
        0.0278,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        0.0278,
        0.0278,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0278,
        0.0278,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3112,
        0.3112,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        0.3112,
        0.3112,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3112,
        0.3112,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
