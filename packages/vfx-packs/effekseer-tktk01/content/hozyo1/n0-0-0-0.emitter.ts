import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "Gloｗ"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.5,
    4.75
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.3922
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.3922
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
