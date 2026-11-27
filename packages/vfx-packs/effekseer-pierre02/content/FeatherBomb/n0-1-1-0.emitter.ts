import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0 "Feather"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    1.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.5,
    4.5
  ],
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
      0.325,
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
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -240.0,
    240.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Feather"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-1-0.effect"
  }
});
