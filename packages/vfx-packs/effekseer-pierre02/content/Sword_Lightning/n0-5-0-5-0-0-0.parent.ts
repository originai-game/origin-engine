import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.5.0 "SparkEmitter"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26,
      "cycles": 2,
      "interval": 0.6
    }
  ],
  "duration": 0.6167,
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-5-0-0-0.emitter"
      }
    }
  ]
});
