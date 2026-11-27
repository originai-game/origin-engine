import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 27,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.983299970626831,
    1.016700029373169
  ],
  "looping": false,
  "maxParticles": 34,
  "rate": 0,
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
      "emitter": ref("base/vfx/effekseer-starter/FireBall/n0-2-1-1-0-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
