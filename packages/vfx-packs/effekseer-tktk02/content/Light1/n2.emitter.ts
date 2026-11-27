import { file } from '@origingame/engine/author';
// Effekseer node 2 "Glow"
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
    0.5833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.2188,
      0.6995
    ],
    [
      0.4531,
      0.8452
    ],
    [
      0.7031,
      0.9407
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
        1.0,
        1.0,
        0.0,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.8513,
        1.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
