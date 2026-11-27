import { file } from '@origingame/engine/author';
// Effekseer node 3 "Charge"
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 120.0,
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.1333,
    0.1333
  ],
  "speed": [
    12.9904,
    51.9615
  ],
  "size": [
    9.0,
    9.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1111
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle02"
  },
  "blend": "additive"
});
