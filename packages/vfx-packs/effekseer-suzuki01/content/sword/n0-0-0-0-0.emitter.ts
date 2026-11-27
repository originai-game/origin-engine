import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0 "Light"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 20.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1333,
    0.1333
  ],
  "speed": [
    6.4,
    6.4
  ],
  "size": [
    4.0,
    4.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      1.0,
      0.6667
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
        0.5732,
        0.4098,
        0.1961
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.5732,
        0.4098,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5732,
        0.4098,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle02"
  },
  "blend": "additive"
});
