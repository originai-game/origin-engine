import { file } from '@origingame/engine/author';
// Effekseer node 2 "moya"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      2.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    2.1866,
    8.7464
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.2737
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.4827
      ]
    ],
    [
      0.6719,
      [
        1.0,
        1.0,
        1.0,
        0.2437
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Fog3001"
  },
  "blend": "subtractive"
});
