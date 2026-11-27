import { file } from '@origingame/engine/author';
// Effekseer node 0.1.7.0 "Particle"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3536,
      1.4142,
      0.3536
    ]
  },
  "spread": 90.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 11,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    1.5,
    2.5
  ],
  "speed": [
    1.75,
    10.053
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre01/Flame/n0-1-7-0-0.emitter"
      }
    }
  ]
});
