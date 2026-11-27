import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.2 "Sword2"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.0,
      3.0,
      3.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.05
    }
  ],
  "duration": 0.9667,
  "looping": false,
  "lifetime": [
    0.3,
    0.3
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.4
    ],
    [
      1.0,
      0.4
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      -1.0,
      0.0
    ]
  },
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
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Sword002"
  },
  "blend": "alpha"
});
