import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "Node"
export default file.particles({
  "maxParticles": 53,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 24.0,
  "duration": 1.0,
  "looping": true,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-0-0-0.emitter"
      }
    }
  ]
});
