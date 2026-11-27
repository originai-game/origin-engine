import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0 "Node"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 50,
      "interval": 0.0333
    }
  ],
  "duration": 1.65,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    5.1962,
    20.7846
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0478,
        0.2195,
        0.251
      ]
    ],
    [
      0.6667,
      [
        0.0,
        0.0478,
        0.2195,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0478,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0,
        0.5356,
        1.0,
        0.7529
      ]
    ],
    [
      0.6667,
      [
        0.0,
        0.5356,
        1.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
