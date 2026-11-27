import { file } from '@origingame/engine/author';
// Effekseer node 4 "Node"
export default file.particles({
  "maxParticles": 8,
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
      "interval": 0.0667
    }
  ],
  "duration": 1.2833,
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
    0.25,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.0,
        0.251
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.5356,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.7529
      ]
    ],
    [
      0.75,
      [
        1.0,
        1.0,
        1.0,
        0.7529
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
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
