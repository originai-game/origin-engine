import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "Node"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      1.0,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0667
    }
  ],
  "duration": 1.95,
  "looping": false,
  "lifetime": [
    0.25,
    0.4167
  ],
  "speed": [
    3.4776,
    13.9104
  ],
  "size": [
    0.5,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.9,
      [
        0.0478,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
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
