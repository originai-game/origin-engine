import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1 "Finish"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 11,
      "cycles": 3,
      "interval": 0.0917
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    3.0,
    3.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-7-3.emitter"
      }
    }
  ]
});
