import { file } from '@origingame/engine/author';
// Effekseer node 11 "Sparks"
export default file.particles({
  "maxParticles": 85,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 200.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    30.0,
    90.0
  ],
  "size": [
    0.3,
    0.7
  ],
  "heightOverLife": [
    [
      0.0,
      10.0
    ],
    [
      1.0,
      10.0
    ]
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
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/orb_colored"
  },
  "blend": "additive"
});
