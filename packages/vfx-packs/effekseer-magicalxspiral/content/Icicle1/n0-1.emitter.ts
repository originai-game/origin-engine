import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Node"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      1.0,
      0.5
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    3.6742,
    14.6969
  ],
  "size": [
    0.125,
    0.375
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.5356,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.5356,
        1.0,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle2"
  },
  "blend": "additive"
});
