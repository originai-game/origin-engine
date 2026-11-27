import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.0 "Glow1"
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
    0.1667,
    0.1667
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
      0.1
    ],
    [
      0.1094,
      0.2948
    ],
    [
      0.2188,
      0.459
    ],
    [
      0.3281,
      0.5955
    ],
    [
      0.4531,
      0.7214
    ],
    [
      0.5781,
      0.8192
    ],
    [
      0.7031,
      0.8933
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.8681,
        0.8055,
        0.2188
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.7464,
        0.6337,
        0.375
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.6346,
        0.4841,
        0.4688
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.5326,
        0.3562,
        0.5
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.4403,
        0.2493,
        0.2812
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.3574,
        0.1628,
        0.125
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
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
