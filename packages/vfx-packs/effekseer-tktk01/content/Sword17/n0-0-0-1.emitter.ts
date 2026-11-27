import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "glow"
export default file.particles({
  "maxParticles": 18,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 14,
      "interval": 0.0167
    }
  ],
  "duration": 0.2333,
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
    0.3889,
    0.3889
  ],
  "heightOverLife": [
    [
      0.0,
      5.7143
    ],
    [
      1.0,
      5.7143
    ]
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
        0.0104,
        0.2195,
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
