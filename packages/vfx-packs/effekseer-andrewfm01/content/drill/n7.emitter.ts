import { file } from '@origingame/engine/author';
// Effekseer node 7 "Bright Sparks"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 30.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    150.0,
    330.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      3.75
    ],
    [
      1.0,
      16.5
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
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
