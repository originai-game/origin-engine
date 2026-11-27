import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0 "Distortion"
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
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    25.0,
    25.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.1094,
      0.2165
    ],
    [
      0.2188,
      0.3989
    ],
    [
      0.3281,
      0.5506
    ],
    [
      0.4531,
      0.6904
    ],
    [
      0.5781,
      0.7991
    ],
    [
      0.7031,
      0.8815
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
      -1.0,
      0.0
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
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.1255
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.02
});
