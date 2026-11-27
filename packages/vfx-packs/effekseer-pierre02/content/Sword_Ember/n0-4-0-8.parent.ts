import { file } from '@origingame/engine/author';
// Effekseer node 0.4.0 "FireEmitter"
export default file.particles({
  "maxParticles": 49,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.7496,
      3.4983,
      0.001
    ]
  },
  "spread": 110.4393,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6667,
    1.1667
  ],
  "speed": [
    1.9515,
    11.998
  ],
  "speedOverLife": [
    [
      0.0,
      0.4746
    ],
    [
      0.25,
      0.8999
    ],
    [
      0.5,
      1.1334
    ],
    [
      0.75,
      1.1667
    ],
    [
      1.0,
      1.0041
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-4-0-8.emitter"
      }
    }
  ]
});
