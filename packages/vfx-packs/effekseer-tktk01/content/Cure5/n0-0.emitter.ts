import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "回復中心もや"
export default file.particles({
  "maxParticles": 1875,
  "shape": {
    "kind": "sphere",
    "radius": 11.0
  },
  "spread": 180.0,
  "rate": 6000.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    0.866,
    3.4641
  ],
  "size": [
    5.0,
    5.0
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
        0.4098,
        0.4205,
        0.8872,
        0.098
      ]
    ],
    [
      0.5,
      [
        0.6661,
        0.4907,
        0.9096,
        0.049
      ]
    ],
    [
      1.0,
      [
        0.9914,
        0.5668,
        0.9323,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
