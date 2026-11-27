import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "Dark"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 11,
      "interval": 0.1333
    }
  ],
  "duration": 1.35,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.25,
    6.25
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.5469,
      0.8725
    ],
    [
      1.0,
      0.72
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ],
    [
      0.15,
      [
        1.0,
        0.8751,
        0.7519,
        0.502
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.8751,
        0.7519,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "subtractive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor_Single/n0-0-0-3.effect"
  }
});
