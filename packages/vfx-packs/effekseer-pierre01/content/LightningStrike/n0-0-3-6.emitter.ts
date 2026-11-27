import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.6 "Burst2"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 25
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.0667,
    0.0667
  ],
  "speed": [
    30.0,
    30.0
  ],
  "size": [
    28.0,
    28.0
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
        0.7519,
        0.3587,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        0.7519,
        0.3587,
        1.0,
        0.1098
      ]
    ],
    [
      0.25,
      [
        0.7519,
        0.3587,
        1.0,
        0.1882
      ]
    ],
    [
      0.375,
      [
        0.7519,
        0.3587,
        1.0,
        0.2353
      ]
    ],
    [
      0.5,
      [
        0.7519,
        0.3587,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.3587,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Burst_1"
  },
  "blend": "additive"
});
