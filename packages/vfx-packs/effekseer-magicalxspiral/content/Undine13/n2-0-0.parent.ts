import { file } from '@origingame/engine/author';
// Effekseer node 2 "Node"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 35,
      "interval": 0.0833
    }
  ],
  "duration": 2.85,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
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
        "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/n2-0-0.emitter"
      }
    }
  ]
});
