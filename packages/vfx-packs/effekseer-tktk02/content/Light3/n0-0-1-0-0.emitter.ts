import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0 "CoreMain"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 12.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.0781,
      0.6318
    ],
    [
      0.1719,
      0.7559
    ],
    [
      0.2656,
      0.8475
    ],
    [
      0.375,
      0.9198
    ],
    [
      0.5781,
      0.9812
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.5,
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
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-0-0.effect"
  }
});
