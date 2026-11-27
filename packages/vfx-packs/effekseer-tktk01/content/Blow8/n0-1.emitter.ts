import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "グロー画像"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 9,
      "interval": 0.1167
    }
  ],
  "duration": 0.95,
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
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7388
    ],
    [
      0.1719,
      0.4988
    ],
    [
      0.2656,
      0.3275
    ],
    [
      0.3594,
      0.2129
    ],
    [
      0.5156,
      0.1153
    ],
    [
      0.8594,
      0.0631
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
        0.9828,
        0.9828,
        0.9828,
        1.0
      ]
    ],
    [
      0.0625,
      [
        0.9558,
        0.8297,
        0.5905,
        0.9577
      ]
    ],
    [
      0.125,
      [
        0.9292,
        0.6906,
        0.304,
        0.9154
      ]
    ],
    [
      0.1875,
      [
        0.903,
        0.5652,
        0.1169,
        0.8731
      ]
    ],
    [
      0.25,
      [
        0.8771,
        0.4535,
        0.0208,
        0.8308
      ]
    ],
    [
      0.4219,
      [
        0.8083,
        0.2137,
        0.0,
        0.7144
      ]
    ],
    [
      0.5938,
      [
        0.7425,
        0.0675,
        0.0,
        0.598
      ]
    ],
    [
      1.0,
      [
        0.5989,
        0.007,
        0.0,
        0.337
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
