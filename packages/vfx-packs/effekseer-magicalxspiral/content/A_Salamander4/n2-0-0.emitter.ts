import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0 "Line1"
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
      "count": 1,
      "cycles": 5,
      "interval": 0.0833
    }
  ],
  "duration": 0.35,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.3
    ],
    [
      1.0,
      0.3
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0478,
        1.0
      ]
    ]
  ],
  "angularVelocity": [
    120.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
