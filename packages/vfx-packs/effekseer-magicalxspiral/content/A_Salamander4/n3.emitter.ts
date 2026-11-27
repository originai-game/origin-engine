import { file } from '@origingame/engine/author';
// Effekseer node 3 "Light1"
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
    0.5833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    15.0,
    15.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ],
    [
      0.0286,
      [
        1.0,
        0.2195,
        0.0,
        0.502
      ]
    ],
    [
      0.7143,
      [
        1.0,
        0.2195,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle3"
  },
  "blend": "additive"
});
