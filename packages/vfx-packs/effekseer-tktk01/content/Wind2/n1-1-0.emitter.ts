import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0 "破裂"
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
    0.5,
    0.5
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
      0.3333
    ],
    [
      0.0781,
      0.5091
    ],
    [
      0.1719,
      0.6745
    ],
    [
      0.2656,
      0.7966
    ],
    [
      0.375,
      0.893
    ],
    [
      0.4688,
      0.9438
    ],
    [
      0.5781,
      0.975
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
      -0.866,
      -0.5
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.2195,
        1.0
      ]
    ],
    [
      0.0781,
      [
        0.2195,
        1.0,
        0.2195,
        0.7364
      ]
    ],
    [
      0.1719,
      [
        0.2195,
        1.0,
        0.2195,
        0.4882
      ]
    ],
    [
      0.2656,
      [
        0.2195,
        1.0,
        0.2195,
        0.3051
      ]
    ],
    [
      0.375,
      [
        0.2195,
        1.0,
        0.2195,
        0.1605
      ]
    ],
    [
      0.4688,
      [
        0.2195,
        1.0,
        0.2195,
        0.0844
      ]
    ],
    [
      0.5781,
      [
        0.2195,
        1.0,
        0.2195,
        0.0375
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    240.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Wind"
  },
  "blend": "additive"
});
