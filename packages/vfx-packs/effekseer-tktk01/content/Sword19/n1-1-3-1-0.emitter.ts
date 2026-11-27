import { file } from '@origingame/engine/author';
// Effekseer node 1.1.3.1.0 "giow"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    3.3333,
    7.3333
  ],
  "speedOverLife": [
    [
      0.0,
      0.7016
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      1.4238
    ]
  ],
  "size": [
    0.6667,
    0.6667
  ],
  "heightOverLife": [
    [
      0.0,
      5.06
    ],
    [
      1.0,
      5.06
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      -1.0,
      0.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.2195,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
