import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 1,
      "cycles": 2,
      "interval": 0.16670000553131104,
      "time": 0
    }
  ],
  "duration": 0.18330000340938568,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 4,
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
      "emitter": ref("base/vfx/effekseer-starter/Claw3/n0-1-0-2-0-0-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
