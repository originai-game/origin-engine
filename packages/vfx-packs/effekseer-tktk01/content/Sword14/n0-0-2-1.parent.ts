import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "*衝撃生業用"
export default file.particles({
  "maxParticles": 1000,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 800
    }
  ],
  "duration": 0.0167,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-2-1.emitter"
      }
    }
  ]
});
