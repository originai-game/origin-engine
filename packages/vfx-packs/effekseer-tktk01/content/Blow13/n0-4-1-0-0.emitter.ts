import { file } from '@origingame/engine/author';
// Effekseer node 0.4.1.0.0 "粒子"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    6.0,
    12.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    0.2,
    0.3
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1094,
      0.7835
    ],
    [
      0.2188,
      0.6011
    ],
    [
      0.3281,
      0.4494
    ],
    [
      0.4531,
      0.3096
    ],
    [
      0.5781,
      0.2009
    ],
    [
      0.7031,
      0.1185
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
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
