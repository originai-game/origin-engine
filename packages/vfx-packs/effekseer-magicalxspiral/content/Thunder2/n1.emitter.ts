import { file } from '@origingame/engine/author';
// Effekseer node 1 "BG2"
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
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.1719,
      0.2509
    ],
    [
      0.5312,
      0.4574
    ],
    [
      0.6719,
      0.5646
    ],
    [
      0.8438,
      0.7519
    ],
    [
      0.9219,
      0.8651
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
        0.0,
        0.2195,
        0.0
      ]
    ],
    [
      0.1111,
      [
        1.0,
        0.0,
        0.2195,
        0.3765
      ]
    ],
    [
      0.7778,
      [
        1.0,
        0.0,
        0.2195,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle3"
  },
  "blend": "additive"
});
