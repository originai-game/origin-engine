import { file } from '@origingame/engine/author';
// Effekseer node 2 "星型"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "box",
    "halfExtents": [
      20.0,
      40.0,
      20.0
    ]
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.75,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    30.0,
    60.0
  ],
  "size": [
    5.0,
    15.0
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
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
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
