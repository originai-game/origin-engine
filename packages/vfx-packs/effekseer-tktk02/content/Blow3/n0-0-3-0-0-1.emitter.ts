import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.0.0.1 "particleGlow"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 960.0,
  "duration": 0.0208,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.5,
    2.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2195,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.0478,
        0.2195,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
