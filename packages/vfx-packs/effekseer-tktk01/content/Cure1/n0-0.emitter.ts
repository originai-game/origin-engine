import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "光メイン上"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.0,
    2.0
  ],
  "size": [
    0.75,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.8592,
        0.9075,
        0.2011,
        0.7843
      ]
    ],
    [
      0.25,
      [
        0.8933,
        0.9302,
        0.3392,
        0.6373
      ]
    ],
    [
      0.5,
      [
        0.9281,
        0.9532,
        0.5174,
        0.4902
      ]
    ],
    [
      0.75,
      [
        0.9637,
        0.9764,
        0.7372,
        0.3431
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1961
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
