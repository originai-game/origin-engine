import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.2.0.0.2 "Distortion"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.75,
    1.25
  ],
  "speedOverLife": [
    [
      0.0,
      4.3039
    ],
    [
      0.2344,
      1.8585
    ],
    [
      0.3438,
      1.052
    ],
    [
      0.4531,
      0.4735
    ],
    [
      0.625,
      0.025
    ],
    [
      0.8438,
      0.2683
    ],
    [
      0.875,
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
        0.0627
      ]
    ],
    [
      0.4286,
      [
        1.0,
        1.0,
        1.0,
        0.0627
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
