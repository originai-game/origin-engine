import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -90,
    90
  ],
  "bursts": [
    {
      "count": 22,
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
  "drag": 2.5999999046325684,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.399999976158142
  ],
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "rotation": [
    0,
    45
  ],
  "shape": {
    "kind": "sphere",
    "radius": 6,
    "surface": false
  },
  "size": [
    16,
    30
  ],
  "sizeOverLife": [
    [
      0,
      0.20000000298023224
    ],
    [
      0.15000000596046448,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    160,
    420
  ],
  "spread": 180,
  "texture": ref("@styles/shared/fx/sparkle.png")
});
