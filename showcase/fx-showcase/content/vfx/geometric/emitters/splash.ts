import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -120,
    120
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.6499999761581421,
    0.7300000190734863,
    0.7799999713897705,
    1
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -5,
    0
  ],
  "lifetime": [
    0.4000000059604645,
    1.100000023841858
  ],
  "looping": false,
  "maxParticles": 20,
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
    "angleDegrees": 45,
    "radius": 0.11999999731779099
  },
  "size": [
    0.029999999329447746,
    0.05999999865889549
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.6499999761581421,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    1.2999999523162842,
    2.5
  ]
});
