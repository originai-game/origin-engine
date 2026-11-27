import { file } from '@origingame/engine/author';
// Effekseer node 6 "Sparks"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    75.0,
    165.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      2.875
    ],
    [
      1.0,
      9.5
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
        0.0,
        0.0278,
        0.4368,
        1.0
      ]
    ],
    [
      0.25,
      [
        0.0,
        0.0278,
        0.4368,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0278,
        0.4368,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2195,
        0.687,
        1.0
      ]
    ],
    [
      0.25,
      [
        0.2158,
        0.2195,
        0.687,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2195,
        0.687,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
