import { file } from '@origingame/engine/author';
// Effekseer node 1 "中心光"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "sphere",
    "radius": 2.0
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    1.299,
    5.1962
  ],
  "size": [
    3.0,
    3.0
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
        0.3922
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
