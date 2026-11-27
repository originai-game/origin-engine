import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "Node"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "sphere",
    "radius": 6.0
  },
  "spread": 180.0,
  "rate": 6000.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    3.3333,
    3.3333
  ],
  "speed": [
    0.0212,
    0.0849
  ],
  "size": [
    0.1,
    0.4
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.4098,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.4098,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.75,
      [
        0.4098,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      1.0,
      [
        0.4098,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.6661,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.6661,
        1.0,
        1.0,
        0.498
      ]
    ],
    [
      0.75,
      [
        0.6661,
        1.0,
        1.0,
        0.498
      ]
    ],
    [
      1.0,
      [
        0.6661,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
