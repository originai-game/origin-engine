import { file } from '@origingame/engine/author';
// Effekseer node 3.1 "Glow"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
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
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.1094,
      0.3732
    ],
    [
      0.2188,
      0.5191
    ],
    [
      0.3281,
      0.6405
    ],
    [
      0.4531,
      0.7523
    ],
    [
      0.5781,
      0.8393
    ],
    [
      0.7031,
      0.9052
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
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.0781,
      [
        0.0438,
        0.2011,
        1.0,
        0.2261
      ]
    ],
    [
      0.1667,
      [
        0.0395,
        0.1813,
        1.0,
        0.4601
      ]
    ],
    [
      0.6667,
      [
        0.0196,
        0.09,
        1.0,
        0.3346
      ]
    ],
    [
      0.8281,
      [
        0.0147,
        0.0677,
        1.0,
        0.1517
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
