import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1 "Distortion"
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
      "count": 10
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.0781,
      0.3373
    ],
    [
      0.1719,
      0.5606
    ],
    [
      0.2656,
      0.7254
    ],
    [
      0.375,
      0.8556
    ],
    [
      0.4688,
      0.9241
    ],
    [
      0.5781,
      0.9662
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
        1.0,
        0.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        0.0,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -30.0,
    30.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion"
  },
  "blend": "additive",
  "distortion": 0.1
});
