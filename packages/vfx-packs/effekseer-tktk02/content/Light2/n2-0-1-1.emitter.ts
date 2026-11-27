import { file } from '@origingame/engine/author';
// Effekseer node 2.0.1.1 "LineColor"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0833,
    5.25
  ],
  "heightOverLife": [
    [
      0.0,
      0.3125
    ],
    [
      1.0,
      0.3125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
