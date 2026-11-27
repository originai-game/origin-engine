import { file } from '@origingame/engine/author';
// Effekseer node 0.7.3.0.0.0.2 "SparkEmitter"
export default file.particles({
  "maxParticles": 30,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.25,
      6.75,
      2.25
    ]
  },
  "spread": 67.0102,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 3,
      "interval": 0.1083
    }
  ],
  "duration": 0.2333,
  "looping": false,
  "lifetime": [
    0.6667,
    1.3333
  ],
  "speed": [
    51.8477,
    77.2573
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-3-0-0-0-2-0-0.emitter"
      }
    }
  ],
  "gravity": [
    -11.8825,
    -70.9808,
    -37.1689
  ]
});
