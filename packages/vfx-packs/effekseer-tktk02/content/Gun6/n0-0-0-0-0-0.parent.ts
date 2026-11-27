import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "LineNull"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 4,
      "interval": 0.0167
    }
  ],
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-0-0-0-0.emitter"
      }
    }
  ]
});
