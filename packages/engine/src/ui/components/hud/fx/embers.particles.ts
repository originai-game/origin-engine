import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 0.4000000059604645,
  "duration": 1.600000023841858,
  "gravity": [
    0,
    30,
    0
  ],
  "lifetime": [
    0.8999999761581421,
    1.2000000476837158
  ],
  "looping": false,
  "maxParticles": 48,
  "rate": 14,
  "round": true,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      1,
      0
    ]
  },
  "size": [
    3,
    5
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0.20000000298023224
    ]
  ],
  "speed": [
    40,
    110
  ],
  "spread": 28
});
