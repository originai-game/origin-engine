import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.15 "Flash2"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.0917
    }
  ],
  "duration": 0.2,
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
    76.65,
    76.65
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
        0.3537,
        0.3787,
        0.8751,
        0.0
      ]
    ],
    [
      0.0469,
      [
        0.3537,
        0.3787,
        0.8751,
        0.3925
      ]
    ],
    [
      0.0857,
      [
        0.3537,
        0.3787,
        0.8751,
        0.6884
      ]
    ],
    [
      1.0,
      [
        0.3537,
        0.3787,
        0.8751,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
