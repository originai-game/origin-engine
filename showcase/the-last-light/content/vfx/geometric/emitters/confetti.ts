import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -180,
    180
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 20,
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
  "lifetime": [
    1,
    1.899999976158142
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.10000000149011612
  },
  "size": [
    0.03500000014901161,
    0.07500000298023224
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.800000011920929,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    1,
    2
  ]
});
