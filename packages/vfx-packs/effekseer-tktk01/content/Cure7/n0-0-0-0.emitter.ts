import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "星"
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
    0.375,
    0.375
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.1961
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.7843
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1961
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
