import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0 "Smoke"
export default file.particles({
  "maxParticles": 244,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      5.0,
      5.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 48,
      "interval": 0.0417
    }
  ],
  "duration": 1.975,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    4.3301,
    17.3205
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.4375
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.8751,
        0.7519,
        1.0,
        0.0784
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.7519,
        1.0,
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-0-0.effect"
  }
});
