import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.1 "Debris"
export default file.particles({
  "maxParticles": 207,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.5,
      0.001,
      2.5
    ]
  },
  "spread": 49.6845,
  "rate": 900.0,
  "duration": 0.0367,
  "looping": false,
  "lifetime": [
    0.3333,
    1.3333
  ],
  "speed": [
    13.9104,
    20.8926
  ],
  "size": [
    1.25,
    1.25
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
        0.0104,
        0.0104,
        0.0104,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.0104,
        0.0104,
        0.0104,
        1.0
      ]
    ],
    [
      0.7,
      [
        0.0104,
        0.0104,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0104,
        0.0104,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -360.0,
    360.0
  ],
  "gravity": [
    0.0,
    -27.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Glass"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-1-1.effect"
  }
});
