import { file } from '@origingame/engine/author';
// Effekseer node 4 "Particle"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
  },
  "spread": 180.0,
  "rate": 120.0,
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    12.9904,
    51.9615
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.1094,
      [
        0.775,
        1.0,
        0.775,
        0.8906
      ]
    ],
    [
      0.2344,
      [
        0.5557,
        1.0,
        0.5557,
        0.7656
      ]
    ],
    [
      0.3594,
      [
        0.3754,
        1.0,
        0.3754,
        0.6406
      ]
    ],
    [
      0.4844,
      [
        0.2329,
        1.0,
        0.2329,
        0.5156
      ]
    ],
    [
      0.6094,
      [
        0.1264,
        1.0,
        0.1264,
        0.3906
      ]
    ],
    [
      0.7344,
      [
        0.0541,
        1.0,
        0.0541,
        0.2656
      ]
    ],
    [
      1.0,
      [
        0.0,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle02"
  },
  "blend": "additive"
});
