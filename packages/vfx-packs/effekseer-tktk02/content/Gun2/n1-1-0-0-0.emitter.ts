import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0 "Fog"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0333
    }
  ],
  "duration": 0.3167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.75,
    0.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.1562,
      [
        0.122,
        0.4205,
        1.0,
        0.5208
      ]
    ],
    [
      0.3,
      [
        0.0589,
        0.3278,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.0104,
        0.2195,
        1.0,
        0.7143
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.1166,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/fog2"
  },
  "blend": "additive"
});
