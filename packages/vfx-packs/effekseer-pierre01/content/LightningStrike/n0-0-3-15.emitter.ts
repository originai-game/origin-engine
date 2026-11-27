import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.15 "Flash2"
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
      "interval": 0.0583
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.3333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    109.5,
    109.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0731
    ],
    [
      0.25,
      0.2524
    ],
    [
      0.5,
      0.4666
    ],
    [
      0.75,
      0.7158
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
        0.8751,
        0.3587,
        0.5356,
        0.0
      ]
    ],
    [
      0.0469,
      [
        0.8751,
        0.3587,
        0.5356,
        0.2616
      ]
    ],
    [
      0.0857,
      [
        0.8751,
        0.3587,
        0.5356,
        0.4589
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.3587,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
