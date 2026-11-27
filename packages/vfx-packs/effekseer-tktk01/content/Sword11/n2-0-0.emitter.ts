import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0 "煙"
export default file.particles({
  "maxParticles": 3125,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 59999.9961,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.908,
    1.908
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3358
    ],
    [
      0.0781,
      1.0353
    ],
    [
      0.0938,
      1.0361
    ],
    [
      0.9844,
      0.0059
    ],
    [
      1.0,
      0.0122
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
        0.7083,
        0.3838,
        0.1062,
        0.3922
      ]
    ],
    [
      0.2031,
      [
        0.7083,
        0.3838,
        0.1062,
        0.2477
      ]
    ],
    [
      0.4219,
      [
        0.7083,
        0.3838,
        0.1062,
        0.1419
      ]
    ],
    [
      0.6562,
      [
        0.7083,
        0.3838,
        0.1062,
        0.069
      ]
    ],
    [
      1.0,
      [
        0.7083,
        0.3838,
        0.1062,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
