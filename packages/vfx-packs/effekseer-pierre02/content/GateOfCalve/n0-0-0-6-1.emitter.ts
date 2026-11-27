import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.6.1 "Flash"
export default file.particles({
  "maxParticles": 394,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 7.5,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.2,
    0.4667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
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
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.5356,
        0.2195,
        0.1625
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.5356,
        0.2195,
        0.25
      ]
    ],
    [
      0.5625,
      [
        1.0,
        0.5356,
        0.2195,
        0.2625
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.5356,
        0.2195,
        0.2
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
