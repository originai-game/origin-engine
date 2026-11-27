import { file } from '@origingame/engine/author';
// Effekseer node 1.4 "*Z軸ランダム"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.1042,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-1.emitter"
      }
    }
  ]
});
