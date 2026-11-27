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
      0.699999988079071,
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
  "drag": 0.20000000298023224,
  "duration": 1.600000023841858,
  "gravity": [
    0,
    240,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    1.7999999523162842
  ],
  "looping": false,
  "maxParticles": 96,
  "rotation": [
    0,
    45
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      1,
      0
    ]
  },
  "size": [
    10,
    22
  ],
  "sizeOverLife": [
    [
      0,
      0.30000001192092896
    ],
    [
      0.20000000298023224,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    10,
    40
  ],
  "spread": 180,
  "texture": ref("@styles/shared/fx/sparkle.png")
});
