import { file } from '@origingame/engine/author';
// Effekseer node 0.7.3.0.0.0.1.0 "Light"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.1245,
      0.0346
    ]
  },
  "spread": 0.5286,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    28.1475,
    45.0141
  ],
  "speedOverLife": [
    [
      0.0,
      0.899
    ],
    [
      0.2656,
      1.3818
    ],
    [
      0.3906,
      1.4445
    ],
    [
      0.5312,
      1.3783
    ],
    [
      0.7812,
      0.9294
    ],
    [
      0.7969,
      0.6254
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    9.0,
    9.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.625
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.231,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.231,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        0.231,
        0.2195,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.231,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
