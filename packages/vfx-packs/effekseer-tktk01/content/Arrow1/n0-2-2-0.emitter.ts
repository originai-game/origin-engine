import { file } from '@origingame/engine/author';
// Effekseer node 0.2.2.0 "先端からの飛び散り"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.25,
    0.25
  ],
  "sizeOverLife": [
    [
      0.0,
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
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.0478,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
