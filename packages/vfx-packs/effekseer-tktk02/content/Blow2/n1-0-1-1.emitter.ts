import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.1 "Glow2"
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
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
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
    22.5,
    22.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0667
    ],
    [
      0.1094,
      0.2687
    ],
    [
      0.2188,
      0.439
    ],
    [
      0.3281,
      0.5806
    ],
    [
      0.4531,
      0.7111
    ],
    [
      0.5781,
      0.8125
    ],
    [
      0.7031,
      0.8894
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
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.4647,
        0.1905,
        0.1647
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.3993,
        0.1636,
        0.2824
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.3392,
        0.139,
        0.3529
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2845,
        0.1166,
        0.3765
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.1905,
        0.0781,
        0.0941
      ]
    ],
    [
      0.875,
      [
        1.0,
        0.1511,
        0.0619,
        0.0235
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
