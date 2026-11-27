import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "星グロー"
export default file.particles({
  "maxParticles": 102,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.75,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    1.5,
    3.0
  ],
  "speedOverLife": [
    [
      0.0,
      0.8568
    ],
    [
      1.0,
      1.1815
    ]
  ],
  "size": [
    0.625,
    0.625
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.5,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.3112,
        0.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.4368,
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.586,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
