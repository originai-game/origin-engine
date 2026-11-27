import { file } from '@origingame/engine/author';
// Effekseer node 3.6.0.0 "Node"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
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
    12.0,
    12.0
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.7519,
        0.7519,
        1.0
      ]
    ],
    [
      0.7333,
      [
        0.7519,
        0.7519,
        0.7519,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.7519,
        0.7519,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
