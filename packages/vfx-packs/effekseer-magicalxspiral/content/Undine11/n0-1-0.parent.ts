import { file } from '@origingame/engine/author';
// Effekseer node 0 "Root"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 59,
      "interval": 0.1667
    }
  ],
  "duration": 9.6833,
  "looping": false,
  "lifetime": [
    0.1667,
    0.1667
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
        "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine11/n0-1-0.emitter"
      }
    }
  ]
});
