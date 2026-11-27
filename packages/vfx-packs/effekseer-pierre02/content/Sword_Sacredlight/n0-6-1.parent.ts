import { file } from '@origingame/engine/author';
// Effekseer node 0.6 "Tracker_5"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 13,
      "interval": 0.0167
    }
  ],
  "duration": 0.2167,
  "looping": false,
  "lifetime": [
    0.2167,
    0.2167
  ],
  "speed": [
    6.0,
    6.0
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-6-1.emitter"
      }
    }
  ]
});
