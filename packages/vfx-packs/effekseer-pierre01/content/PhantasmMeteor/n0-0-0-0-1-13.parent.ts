import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "MotionControl"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    2.0833,
    2.0833
  ],
  "speed": [
    4.8904,
    4.8904
  ],
  "speedOverLife": [
    [
      0.0,
      0.6016
    ],
    [
      0.4688,
      0.4433
    ],
    [
      0.4844,
      2.3823
    ],
    [
      0.9219,
      2.3823
    ],
    [
      0.9531,
      24.5653
    ],
    [
      0.9688,
      24.5653
    ],
    [
      0.9844,
      24.2705
    ],
    [
      1.0,
      0.0
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
        "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-1-13.emitter"
      }
    }
  ]
});
