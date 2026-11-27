import { file } from '@origingame/engine/author';
// Effekseer node 5.0.0.0 "残像"
export default file.particles({
  "maxParticles": 155,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.4758,
    1.4758
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
        0.1961
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/ezziglow"
  },
  "blend": "additive"
});
