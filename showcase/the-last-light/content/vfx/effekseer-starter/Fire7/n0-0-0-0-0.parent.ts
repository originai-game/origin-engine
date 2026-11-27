import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 10,
      "cycles": 6,
      "interval": 0.08330000191926956,
      "time": 0
    }
  ],
  "duration": 0.4332999885082245,
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 63,
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
      "emitter": ref("base/vfx/effekseer-starter/Fire7/n0-0-0-0-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
