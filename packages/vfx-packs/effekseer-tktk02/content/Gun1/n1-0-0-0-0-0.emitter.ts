import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0.0 "shotCore"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.6
    }
  ],
  "duration": 0.6167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    600.0,
    600.0
  ],
  "speedOverLife": [
    [
      0.0,
      16.3905
    ],
    [
      0.0469,
      9.2192
    ],
    [
      0.0625,
      7.188
    ],
    [
      0.0781,
      5.4661
    ],
    [
      0.1094,
      2.9503
    ],
    [
      0.125,
      1.1581
    ],
    [
      0.1406,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    4.0,
    4.0
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
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
