import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    -1300,
    0
  ],
  "lifetime": [
    1,
    1.399999976158142
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    45
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 30,
    "radius": 4
  },
  "size": [
    12,
    20
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.10000000149011612,
      1
    ],
    [
      0.20000000298023224,
      0.20000000298023224
    ],
    [
      0.3499999940395355,
      1
    ],
    [
      0.5,
      0.20000000298023224
    ],
    [
      0.6499999761581421,
      1
    ],
    [
      0.800000011920929,
      0.20000000298023224
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    420,
    620
  ],
  "texture": ref("@styles/shared/fx/sparkle.png")
});
