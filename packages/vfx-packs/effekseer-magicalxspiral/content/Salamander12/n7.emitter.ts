import { file } from '@origingame/engine/author';
// Effekseer node 7 "Node"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0333
    }
  ],
  "duration": 0.65,
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
    0.5,
    1.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.5356,
        0.2195,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.5295,
        0.5356,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.5356,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
