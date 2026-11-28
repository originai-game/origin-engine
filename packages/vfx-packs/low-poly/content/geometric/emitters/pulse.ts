import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.8999999761581421,
    0.2199999988079071,
    0.11999999731779099,
    1
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "heightOverLife": [
    [
      0,
      2
    ],
    [
      1,
      0
    ]
  ],
  "lifetime": [
    0.10000000149011612,
    0.10000000149011612
  ],
  "looping": false,
  "maxParticles": 1,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "shape": {
    "kind": "point"
  },
  "size": [
    0.20000000298023224,
    0.20000000298023224
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    2,
    2
  ]
});
