import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "火尻尾"
export default file.particles({
  "maxParticles": 80,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 64
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.25,
    0.9167
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-0-0-0-0.emitter"
      }
    }
  ]
});
