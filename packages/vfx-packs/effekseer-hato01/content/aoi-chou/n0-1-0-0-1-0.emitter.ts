import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.1.0 "Node"
export default file.particles({
  "maxParticles": 2303,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 8,
      "interval": 0.36
    }
  ],
  "duration": 2.5367,
  "looping": false,
  "lifetime": [
    3.3333,
    5.0
  ],
  "speed": [
    0.7176,
    0.7176
  ],
  "size": [
    0.39,
    0.39
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.021,
        0.1701,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.021,
        0.1701,
        1.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3066,
        0.4878,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3066,
        0.4878,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-hato01/textures/particle2"
  },
  "blend": "additive"
});
