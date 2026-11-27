import { file } from '@origingame/engine/author';
// Effekseer node 0 "連撃処理"
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 9,
      "interval": 0.1167
    }
  ],
  "duration": 0.95,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Blow8/n0-0.emitter"
      }
    }
  ]
});
