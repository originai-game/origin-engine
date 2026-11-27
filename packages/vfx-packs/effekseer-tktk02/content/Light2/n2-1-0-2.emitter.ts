import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.2 "LineGlow"
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
    9.9167,
    13.4167
  ],
  "heightOverLife": [
    [
      0.0,
      0.125
    ],
    [
      1.0,
      0.125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.5356,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.5356,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
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
