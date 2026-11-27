import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.1 "CoreColor"
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
    3.0,
    5.0
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
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.5,
      [
        0.8671,
        1.0,
        0.0055,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.5,
      [
        0.9323,
        1.0,
        0.0254,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-0-1.effect"
  }
});
