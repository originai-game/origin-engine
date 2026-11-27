import { file } from '@origingame/engine/author';
// Effekseer node 1.4 "Glow"
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
    12.5,
    12.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      0.1094,
      0.5299
    ],
    [
      0.2188,
      0.6394
    ],
    [
      0.4531,
      0.8143
    ],
    [
      0.7031,
      0.9289
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
        0.5356,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4134,
        0.032,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.3587,
        0.0254,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
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
