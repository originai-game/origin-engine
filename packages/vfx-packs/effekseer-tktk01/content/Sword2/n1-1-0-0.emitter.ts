import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0 "Node"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 18000.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.2667,
    0.2667
  ],
  "speed": [
    0.3654,
    1.4614
  ],
  "size": [
    0.45,
    0.45
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1667
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
        0.1961
      ]
    ],
    [
      0.2344,
      [
        1.0,
        0.613,
        0.613,
        0.1501
      ]
    ],
    [
      0.3594,
      [
        1.0,
        0.448,
        0.448,
        0.1256
      ]
    ],
    [
      0.4844,
      [
        1.0,
        0.3107,
        0.3107,
        0.1011
      ]
    ],
    [
      0.6094,
      [
        1.0,
        0.2003,
        0.2003,
        0.0766
      ]
    ],
    [
      0.7344,
      [
        1.0,
        0.1156,
        0.1156,
        0.0521
      ]
    ],
    [
      0.8594,
      [
        1.0,
        0.0555,
        0.0555,
        0.0276
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0152,
        0.0152,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
