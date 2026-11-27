import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "ｉｎｔｅｒｖａｌ"
export default file.particles({
  "maxParticles": 18,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 14,
      "interval": 0.0333
    }
  ],
  "duration": 0.45,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Light2/n0-0-0-1-0-0-1.emitter"
      }
    }
  ]
});
