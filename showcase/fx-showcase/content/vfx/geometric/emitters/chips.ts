import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -90,
    90
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 14,
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
    0.25,
    0.5
  ],
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 65,
    "radius": 0.03999999910593033
  },
  "size": [
    0.05999999865889549,
    0.11999999731779099
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
    1.399999976158142,
    2.5999999046325684
  ]
});
