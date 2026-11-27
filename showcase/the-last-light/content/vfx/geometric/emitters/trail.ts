import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.8999999761581421,
    0.2199999988079071,
    0.11999999731779099,
    1
  ],
  "duration": 2,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.30000001192092896,
    0.6000000238418579
  ],
  "maxParticles": 16,
  "rate": 20,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "shape": {
    "kind": "point"
  },
  "size": [
    0.054999999701976776,
    0.054999999701976776
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
    0.20000000298023224,
    0.20000000298023224
  ]
});
