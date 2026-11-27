import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1 "火の尻尾"
export default file.particles({
  "maxParticles": 438,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.5833,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    10.125,
    10.125
  ],
  "speedOverLife": [
    [
      0.0,
      2.0
    ],
    [
      0.4844,
      2.0
    ],
    [
      0.5156,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    3.375,
    3.375
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
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
        0.098
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
