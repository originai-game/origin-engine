import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "duration": 0.10419999808073044,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 32,
  "rate": 240,
  "shape": {
    "kind": "point"
  },
  "size": [
    0,
    0
  ],
  "speed": [
    0,
    0
  ],
  "subEmitters": [
    {
      "emitter": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-1.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
