import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "光追跡"
export default file.particles({
  "maxParticles": 825,
  "shape": {
    "kind": "sphere",
    "radius": 0.5
  },
  "spread": 180.0,
  "rate": 240.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    1.299,
    5.1962
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "alpha"
});
