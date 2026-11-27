import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "BaseLight"
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
    9.0,
    9.0
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
        0.6661,
        0.2845,
        0.0
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.6661,
        0.2845,
        0.251
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.6661,
        0.2845,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.6661,
        0.2845,
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
