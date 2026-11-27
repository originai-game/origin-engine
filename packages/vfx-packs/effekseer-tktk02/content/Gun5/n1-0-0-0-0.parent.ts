import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0 "6Ring"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.25,
    0.5833
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0-0.emitter"
      }
    }
  ]
});
