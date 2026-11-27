import { file } from '@origingame/engine/author';
// Effekseer node 5.1.0.0.0.2.0.0.2 "Distortion"
export default file.particles({
  "maxParticles": 1875,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 150,
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
    0.9,
    1.5
  ],
  "speedOverLife": [
    [
      0.0,
      5.8771
    ],
    [
      0.1875,
      2.293
    ],
    [
      0.2812,
      1.1016
    ],
    [
      0.375,
      0.3422
    ],
    [
      0.4844,
      0.0023
    ],
    [
      0.6094,
      0.3338
    ],
    [
      0.6406,
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
