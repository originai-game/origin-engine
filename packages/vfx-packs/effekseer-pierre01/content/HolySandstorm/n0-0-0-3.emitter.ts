import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "Particle"
export default file.particles({
  "maxParticles": 178,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.2,
      0.2,
      0.2
    ]
  },
  "spread": 88.727,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 240,
      "interval": 0.0167
    }
  ],
  "duration": 4.0,
  "looping": false,
  "lifetime": [
    1.0,
    1.3333
  ],
  "speed": [
    27.0067,
    27.0599
  ],
  "size": [
    0.09,
    0.31
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.4423,
        0.2195,
        1.0
      ]
    ],
    [
      0.4286,
      [
        0.5356,
        0.4423,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.4423,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -1800.0,
    1800.0
  ],
  "gravity": [
    13.9762,
    0.0,
    52.16
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Hard"
  },
  "blend": "additive"
});
