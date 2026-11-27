import { file } from '@origingame/engine/author';
// Effekseer node 0 "Stars"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "point"
  },
  "spread": 98.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.4283,
    2.2891
  ],
  "speedOverLife": [
    [
      0.0,
      1.0041
    ],
    [
      0.25,
      1.1667
    ],
    [
      0.5,
      1.1334
    ],
    [
      0.75,
      0.8999
    ],
    [
      1.0,
      0.4746
    ]
  ],
  "size": [
    0.3,
    0.3
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.0667,
      [
        0.5356,
        0.5356,
        1.0,
        0.1255
      ]
    ],
    [
      0.1667,
      [
        0.5356,
        0.5356,
        1.0,
        0.1255
      ]
    ],
    [
      0.4375,
      [
        0.5356,
        0.5356,
        1.0,
        0.0634
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
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
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.2969,
      [
        1.0,
        1.0,
        1.0,
        0.2697
      ]
    ],
    [
      0.4375,
      [
        1.0,
        1.0,
        1.0,
        0.1903
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
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
