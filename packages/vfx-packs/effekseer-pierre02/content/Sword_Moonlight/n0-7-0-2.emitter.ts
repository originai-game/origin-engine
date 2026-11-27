import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.2 "Burst2"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.0667,
    0.0667
  ],
  "speed": [
    12.0,
    12.0
  ],
  "size": [
    11.2,
    11.2
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7825
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.125,
      [
        0.2195,
        1.0,
        0.5356,
        0.1098
      ]
    ],
    [
      0.25,
      [
        0.2195,
        1.0,
        0.5356,
        0.1882
      ]
    ],
    [
      0.375,
      [
        0.2195,
        1.0,
        0.5356,
        0.2353
      ]
    ],
    [
      0.5,
      [
        0.2195,
        1.0,
        0.5356,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_1"
  },
  "blend": "additive"
});
