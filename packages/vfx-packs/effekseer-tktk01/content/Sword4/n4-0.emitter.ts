import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "斬撃〆"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    40.0,
    40.0
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    2.0,
    2.0
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
  "heightOverLife": [
    [
      0.0,
      3.0
    ],
    [
      1.0,
      3.0
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
        0.1011,
        0.7445,
        0.8277,
        0.8529
      ]
    ],
    [
      0.2656,
      [
        0.1011,
        0.7445,
        0.8277,
        0.6505
      ]
    ],
    [
      0.7812,
      [
        0.1011,
        0.7445,
        0.8277,
        0.1805
      ]
    ],
    [
      1.0,
      [
        0.1011,
        0.7445,
        0.8277,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
