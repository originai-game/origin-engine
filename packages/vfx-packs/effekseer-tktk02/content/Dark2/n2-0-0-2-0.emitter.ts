import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.2.0 "particleMain"
export default file.particles({
  "maxParticles": 750,
  "shape": {
    "kind": "sphere",
    "radius": 0.875
  },
  "spread": 0.0,
  "rate": 1800.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.2188,
    0.3646
  ],
  "speedOverLife": [
    [
      0.0,
      2.8378
    ],
    [
      0.1719,
      1.9946
    ],
    [
      0.3125,
      1.4159
    ],
    [
      0.5156,
      0.7848
    ],
    [
      0.7344,
      0.3758
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.1458,
    0.1458
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
