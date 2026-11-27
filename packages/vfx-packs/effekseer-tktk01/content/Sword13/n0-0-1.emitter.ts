import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1 "*衝撃生業用"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0167
    }
  ],
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.5,
    0.5
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
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      1.0,
      4.0
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
        0.586,
        1.0,
        1.0,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.586,
        1.0,
        1.0,
        0.1961
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "alpha"
});
