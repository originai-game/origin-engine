import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "BaseLight"
export default file.particles({
  "maxParticles": 263,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 1.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.3333,
    1.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8751,
        0.5605,
        0.0
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.8751,
        0.5605,
        0.9412
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.8751,
        0.5605,
        0.9412
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.5605,
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
  "blend": "alpha"
});
