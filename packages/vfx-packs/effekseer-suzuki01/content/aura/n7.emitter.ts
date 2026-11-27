import { file } from '@origingame/engine/author';
// Effekseer node 7 "Needle2"
export default file.particles({
  "maxParticles": 102,
  "shape": {
    "kind": "sphere",
    "radius": 1.5
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.2667,
    0.2667
  ],
  "speed": [
    1.5,
    1.5
  ],
  "speedOverLife": [
    [
      0.0,
      0.7049
    ],
    [
      0.25,
      0.9438
    ],
    [
      0.5,
      1.0749
    ],
    [
      0.75,
      1.0937
    ],
    [
      1.0,
      1.0023
    ]
  ],
  "size": [
    -0.03,
    0.05
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.7012,
        1.0,
        1.0,
        0.7843
      ]
    ],
    [
      1.0,
      [
        0.7012,
        1.0,
        1.0,
        0.7843
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
