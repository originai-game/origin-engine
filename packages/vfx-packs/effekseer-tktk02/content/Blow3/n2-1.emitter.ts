import { file } from '@origingame/engine/author';
// Effekseer node 2.1 "Glow"
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
    15.0,
    15.0
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
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.147,
        0.6707,
        1.0,
        1.0
      ]
    ],
    [
      0.6562,
      [
        0.0915,
        0.4187,
        1.0,
        0.5156
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
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
