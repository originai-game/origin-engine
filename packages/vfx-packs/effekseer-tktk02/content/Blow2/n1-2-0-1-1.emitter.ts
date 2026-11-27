import { file } from '@origingame/engine/author';
// Effekseer node 1.2.0.1.1 "Distortion"
export default file.particles({
  "maxParticles": 39,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
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
    2.25,
    2.25
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion4"
  },
  "blend": "additive",
  "distortion": 0.1
});
