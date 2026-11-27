import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0.0.0.0.2 "Distortion"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 50,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    1.75,
    2.25
  ],
  "speedOverLife": [
    [
      0.0,
      4.9106
    ],
    [
      0.2188,
      1.9764
    ],
    [
      0.3281,
      0.9869
    ],
    [
      0.4219,
      0.4095
    ],
    [
      0.5625,
      0.0123
    ],
    [
      0.7344,
      0.2906
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.4,
    0.6
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7364
    ],
    [
      0.1719,
      0.4882
    ],
    [
      0.2656,
      0.3051
    ],
    [
      0.375,
      0.1605
    ],
    [
      0.4688,
      0.0844
    ],
    [
      0.5781,
      0.0375
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.251
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
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion"
  },
  "blend": "additive",
  "distortion": 0.004
});
