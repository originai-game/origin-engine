import { file } from '@origingame/engine/author';
// Effekseer node 7 "Node"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0833
    }
  ],
  "duration": 1.6,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    1.9486,
    7.7942
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0,
        1.0,
        1.0
      ]
    ],
    [
      0.8,
      [
        0.2195,
        0.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      0.8,
      [
        1.0,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
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
