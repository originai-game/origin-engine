import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.2.9 "Flash2"
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
      "count": 1,
      "cycles": 3,
      "interval": 0.0667
    }
  ],
  "duration": 0.15,
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
    438.0,
    438.0
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.5212
      ]
    ],
    [
      0.0857,
      [
        1.0,
        1.0,
        1.0,
        0.9143
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
